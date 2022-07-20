use crate::{config::MEMORY_END, mm::address::PhysAddr, up::UPSafeCell};

use super::{address::PhysPageNum, frame_tracker::FrameTracker};

use alloc::vec::Vec;

use lazy_static::*;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,       // 空闲空间的开始页面号 ---\\
    end: usize,           // 空闲空间的结束页面号 ---//
    recycled: Vec<usize>, // 已回收的页面
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;

        // validity check
        // 如果
        // - 待回收的页面号 `大于` 空闲空间的开始页面号，或者
        // - 待回收的页面号位于 `已回收列表` 里
        // 则表明待回收的帧是非法的。
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }

        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

// 帧分配器（记录着分配情况的结构体 StackFrameAllocator 实例）创建在 .bss 里
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    // 将 `内核程序结束的位置（即 linker.ld 的 ekernel 位置）` 到
    // `物理内存的结束的位置（MEMORY_END）` 作为帧可分配的空间。
    // 注意物理内存的起始物理地址为 0x80000000
    // 虽然 MEMORY_END 的值为 0x8080_0000，实际上只有 8MB
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

/// 对外服务的函数
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// 对外服务的函数
/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut ppn0 = Vec::<usize>::new();
    let mut ppn1 = Vec::<usize>::new();

    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();

        println!("{:?}", frame);
        ppn0.push(frame.ppn.0);

        v.push(frame);
    }
    v.clear();

    println!("recycle frames");

    // 下面的分配会重用上面已回收的 frame

    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        ppn1.push(frame.ppn.0);

        v.push(frame);
    }
    drop(v);

    // 比较两个 PPN 列表，它们的值应该相等才对
    ppn0.sort();
    ppn1.sort();
    assert_eq!(ppn0, ppn1);

    // 测试分配超出内存的 frame
    extern "C" {
        fn ekernel();
    }

    let start_num = PhysAddr::from(ekernel as usize).ceil();
    let end_num = PhysAddr::from(MEMORY_END).floor();

    println!("start phy page num: {}", start_num.0);
    println!("end phy page num: {}", end_num.0);

    // 下面的应该能正常分配
    let mut v: Vec<FrameTracker> = Vec::new();
    for _ in start_num.0..end_num.0 {
        let frame = frame_alloc().unwrap();
        v.push(frame);
    }

    // 下面的应该无法分配
    let one_more = frame_alloc();
    assert!(matches!(one_more, None));

    drop(v);

    println!("frame_allocator_test passed!");
}
