use self::memory_set::KERNEL_SPACE;

mod heap_allocator;
pub mod address;
pub mod page_table;
mod frame_tracker;
mod frame_allocator;
pub mod memory_set;

pub fn init() {
    heap_allocator::init_heap();
    // heap_allocator::heap_test(); // 测试

    frame_allocator::init_frame_allocator();
    // frame_allocator::frame_allocator_test(); // 测试

    KERNEL_SPACE.exclusive_access().activate();
    // memory_set::remap_test(); // 测试
}