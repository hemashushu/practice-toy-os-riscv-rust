use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;

use super::{
    address::{PhysPageNum, StepByOne, VirtAddr, VirtPageNum},
    frame_allocator::frame_alloc,
    frame_tracker::FrameTracker,
};

// 页表项的数据结构
//
// | 63 -- 54 | 53 -- 28 | 27 -- 19 | 18 -- 10 | 9 8 | 7 6 5 4 3 2 1 0 |
// | reserved | ppn[2]   | ppn[1]   | ppn[0]   | RSW | D A             | <-- 处理器自动动态设置
// |                                                       G           |
// |                                                         U         | <-- U - U 特权级可访问
// |                                                           X W R   | <-- 执行/写/读 权限
// |                                                                 V | <-- 有效位

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

// 页表数据结构
//
//     |-----------------------------|
// 511 | 10 bits | 44 bits | 10 bits | <-- 页表项
// ... | 10 bits | 44 bits | 10 bits |
// 001 | 10 bits | 44 bits | 10 bits |
//     |-----------------------------|
//
// 多级页表
//
// Virtual Address
//
// |---------------------------------------------|
// |                  39 bits                    |
// | 9 bits | 9 bits | 9 bits | 9 bits | 12 bits |
// | ext    | L2     | L1     | L0     | Offset  |
// |--------|--------|--------|--------|---------|
//            |        |          |           |
//   /--------/     /--/          |           |
//   |              |             |           \--------\
//   |    |-----|   |   |-----|   |   |-----|          |
//   |--> |     |-\ \-> |     |-\ \-> |     | --> PPN  Offset
//        |     | |     |     | |     |     |     ===========
// satp ->|-----| \---> |-----| \---> |-----|     Physical Addr
//         L2 table      L1 table      L0 table
//
// http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/3sv39-implementation-1.html#id7
//
// - 当 V 为 0 的时候，代表当前指针是一个空指针，无法走向下一级节点，即该页表项对应的虚拟地址范围是无效的；
// - 当 V 为1 且 R/W/X 均为 0 时，表示是一个合法的页目录表项，其包含的指针会指向下一级的页表；
//   L2 和 L1 表的 PageTableEntry 应该是这种情况
// - 当 V 为1 且 R/W/X 不全为 0 时，表示是一个合法的页表项，其包含了虚地址对应的物理页号。
//   L0 表的 PageTableEntry 应该是这种情况

/// page table structure
///
/// 注意：
/// 内核程序只需维护（创建\更新）多级页表，
/// 程序在 load/store 内存数据时，CPU 会自动通过多级页表
/// 查询得到实际的物理地址，这一步并不需要内核程序参与（实际上
/// 需要为程序提供第一个页表的物理地址，该地址可以视为程序的内存 root 地址，
/// 把 root 地址写入 satp 寄存器，这就是内核程序所需要做的）
pub struct PageTable {
    root_ppn: PhysPageNum,

    // 注意
    // 这里储存的不是 PageTableEntry，而是直接储存
    // PageTableEntry 对于的 PageTable 了
    // 一个 PageTable 最多由 512 个 Entry，所以这里 frames 也会有 512 项
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    /// 跟据虚拟地址找到对应的 PageTableEntry
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];

            if i == 2 {
                // 最后一级，即 L0
                // 这里并没有检查 V == 1 且 X/W/R 至少又一个不为 0
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                return None;
            }

            // 这里并没有检查 V == 1 且 X/W/R 是否均为 0
            ppn = pte.ppn();
        }
        result
    }

    /// 跟据虚拟地址找到对应的 PageTableEntry，
    /// 如果找不到则创建新的。
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];

            if i == 2 {
                // 最后一级，即 L0
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                // 没找到对应的 PageTableEntry，创建一个下级表
                // 注意这里是创建一个 PageTable
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }

            ppn = pte.ppn();
        }
        result
    }

    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    /// 构建适合赋值给 satp CSR 寄存器的数据（u64）
    pub fn token(&self) -> usize {
        // satp CSR 寄存器的数据结构
        //
        // | 63 --- 60 | 59 --- 44 | 43 --- 0 |
        // | mode      | ASID      | 根 PPN   |
        //
        // 当 MODE 设置为 0 的时候，代表所有访存都被视为物理地址；
        // 而设置为 8 的时候，SV39 分页机制被启用，所有 S/U 特权级的访存被视为
        // 一个 39 位的虚拟地址，它们需要先经过 MMU 的地址转换流程，
        // 如果顺利的话，则会变成一个 56 位的物理地址来访问物理内存
        //
        // http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/3sv39-implementation-1.html#satp-layout
        8usize << 60 | self.root_ppn.0
    }

    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
}

// 由于内核和应用地址空间的隔离， sys_write 不再能够直接访问位于应用空间中的数据，
// 而需要手动查页表才能知道那些数据被放置在哪些物理页帧上并进行访问。
// 为此，页表模块 page_table 提供了将应用地址空间中一个缓冲区转化为
// 在内核空间中能够直接访问的形式的辅助函数：
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        v.push(&ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        start = end_va.into();
    }
    v
}
