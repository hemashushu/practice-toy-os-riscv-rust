use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

use crate::{
    config::{MEMORY_END, MMIO, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    mm::address::StepByOne,
    up::UPSafeCell,
};

use super::{
    address::{PhysAddr, PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::frame_alloc,
    frame_tracker::FrameTracker,
    page_table::{PTEFlags, PageTable, PageTableEntry},
};

/// `内存段`
/// 一段**连续**的 Virtual Page
pub struct MapArea {
    vpn_range: VPNRange,                              // VPN 的开始和结束值
    data_frames: BTreeMap<VirtPageNum, FrameTracker>, // VPN 对应的最终的物理 Page，仅当 MapType 为 Framed 时才使用
    map_type: MapType,                                // 内存的映射方式
    map_perm: MapPermission,                          // 该段内存的访问权限
}

/// 内存的映射方式
/// map type for memory set: identical or framed
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical, // 恒等映射方式
    Framed,    // 3 级页表方式
}

use bitflags::bitflags;

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();

        println!("new map area, virtual page number range: [0x{:x}, 0x{:x})", start_vpn.0, end_vpn.0);

        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) -> usize { // 新增，返回物理页面地址

        // 注：
        // 这里最好检查以下参数 vpn 是否属于 self.vpn_range 之内。

        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }

            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }

        let ppn_clone = ppn.0; // 新增

        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);

        ppn_clone // 新增
    }

    #[allow(unused)]
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }

    pub fn map(&mut self, page_table: &mut PageTable) -> Vec<usize> { // 新增，返回物理页面地址列表
        let mut ppns = Vec::<usize>::new();

        for vpn in self.vpn_range {
            let ppn = self.map_one(page_table, vpn);
            ppns.push(ppn);
        }

        ppns
    }

    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    ///
    /// 将参数 `data` 的数据复制到 `self.vpn_range`。
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];

            dst.copy_from_slice(src);
            start += PAGE_SIZE;

            if start >= len {
                break;
            }

            current_vpn.step();
        }
    }
}

/// memory set structure, controls virtual-memory space
///
/// 一系列 `有关联的不一定连续的逻辑段`
/// 一般就是一个程序对应一个 MemorySet
pub struct MemorySet {
    page_table: PageTable, // 第一个 page table，即 L2 page table
    areas: Vec<MapArea>,   // `内存段` 集合
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

use core::arch::asm;
use lazy_static::lazy_static;
use riscv::register::satp;

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        println!("insert_framed_area (map a kernel-stack in kernel space) 0x{:x}-0x{:x}", start_va.0, end_va.0);

        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) -> Vec<usize> { // 新增，返回物理页面地址列表
        let pnns = map_area.map(&mut self.page_table);

        // 用于加载 app 的二进制数据
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }

        self.areas.push(map_area);

        pnns
    }

    /// Mention that trampoline is not collected by areas.
    ///
    /// 启用了分页机制之后，用户 app trap 需要切换到内核地址空间，以及内核处理完 trap 之后需要切换回到 app 的地址空间，
    /// 要求地址空间的切换不能影响指令的连续执行，即要求应用和内核地址空间在切换地址空间指令附近是平滑的。
    /// 所以需要一个跳板。
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );

        println!("map trampoline, virtual page number: 0x{:x}, physical page number: 0x{:x}",
            TRAMPOLINE,
            strampoline as usize
        );
    }

    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        println!("------ mapping kernel");

        let mut memory_set = Self::new_bare();

        // map trampoline
        memory_set.map_trampoline();

        // map kernel sections

        // println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        // println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        // println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        // println!(
        //     ".bss [{:#x}, {:#x})",
        //     sbss_with_stack as usize, ebss as usize
        // );

        println!("mapping .text section");
        let pnns_text = memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
            pnns_text.first().unwrap(),
            pnns_text.last().unwrap());

        println!("mapping .rodata section");
        let pnns_rodata = memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
            pnns_rodata.first().unwrap(),
            pnns_rodata.last().unwrap());

        println!("mapping .data section");
        let pnns_data = memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
            pnns_data.first().unwrap(),
            pnns_data.last().unwrap());

        println!("mapping .bss section");
        let pnns_bss = memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
            pnns_bss.first().unwrap(),
            pnns_bss.last().unwrap());

        println!("mapping physical memory");
        let pnns_pm = memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
            pnns_pm.first().unwrap(),
            pnns_pm.last().unwrap());

        println!("mapping memory-mapped registers");
        for pair in MMIO {
            let pnns_mmio = memory_set.push(
                MapArea::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
            println!("map to physical page number (identical): 0x{:x} ... 0x{:x}",
                pnns_mmio.first().unwrap(),
                pnns_mmio.last().unwrap());
        }

        memory_set
    }

    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();

        // map trampoline
        memory_set.map_trampoline();

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();

            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // 对于非 Type::Load 类型的 program header 不予理睬

                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();

                let mut map_perm = MapPermission::U; // U 表示用户 app 可访问权限
                let ph_flags = ph.flags();

                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }

                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }

                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }

                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);

                max_end_vpn = map_area.vpn_range.get_end();

                let pnns = memory_set.push(
                    map_area,
                    // 注意当存在一部分零初始化的时候， ph.file_size() 将会小于 ph.mem_size()
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );

                println!("map to physical page number (framed): 0x{:x}, ???, ... 0x{:x}",
                    pnns.first().unwrap(),
                    pnns.last().unwrap());
            }
        }

        // map user stack with U flags

        // 注意
        // 用户应用程序的 “栈” 是由内核在切换应用程序时由 sp 寄存器指定位置的，
        // 作为应用程序本身并不需要操心自己的栈以及堆（虽然目前的 app 仍未用到 heap）如何分配以及位置在哪里
        println!("mapping user application stack");
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE; // guard page

        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let ppns_stack = memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        println!("map to physical page number (framed): 0x{:x}, ???, ... 0x{:x}",
            ppns_stack.first().unwrap(),
            ppns_stack.last().unwrap());

        // map TrapContext
        println!("mapping user application TrapContext");
        let ppns_tc = memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("map to physical page number (framed): 0x{:x}, ???, ... 0x{:x}",
            ppns_tc.first().unwrap(),
            ppns_tc.last().unwrap());

        // 返回的内容不仅仅包括内存空间，
        // 还应该包括用户 app 栈的页面地址
        // 以及应用程序的入口地址等。
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);

            // sfence.vma 指令将快表（TLB）清空
            asm!("sfence.vma");
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
}


#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();

    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();

    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);

    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);

    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);

    println!("remap_test passed!");
}
