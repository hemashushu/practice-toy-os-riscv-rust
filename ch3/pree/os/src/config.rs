// 内核 trap 栈和用户栈的大小都是 8KB
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const MAX_APP_NUM: usize = 4;

pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

// QEMU 的时钟频率, 12.5MHz
pub const CLOCK_FREQ: usize = 12500000;
