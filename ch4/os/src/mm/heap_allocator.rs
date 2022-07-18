use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

// 在 .bss 段开辟一段空间作为 heap 空间
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        let heap_start = HEAP_SPACE.as_ptr() as usize;

        println!(
            "heap start: 0x{:x}, end: 0x{:x}",
            heap_start,
            heap_start + KERNEL_HEAP_SIZE
        );

        HEAP_ALLOCATOR.lock().init(heap_start, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use alloc::string::String;

    extern "C" {
        fn sbss();
        fn ebss();
    }

    // 用于检测变量的指针的位置
    let bss_range = sbss as usize..ebss as usize;

    // 测试 Box
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);

    // 测试 Vec
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }

    for i in 0..500 {
        assert_eq!(v[i], i);
    }

    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);

    // 测试 String
    let mut s = String::from("hello");
    s.push(' ');
    s.push_str("world");

    assert_eq!(s, "hello world");
    drop(s);

    // 测试完毕
    println!("heap_test passed!");
}
