// 内核 trap 栈和用户栈的大小都是 8KB
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; // 192 KB
pub const PAGE_SIZE: usize = 0x1000; // 4KB
pub const PAGE_SIZE_BITS: usize = 0xc; // 12 bits

// 注意物理内存的起始物理地址为 0x80000000（即 2GB 的位置）
// 所以 MEMORY_END 虽然值为 2056 MB，实际代表着内存 8MB 的位置。
// 内核开始和结束位置由 linker.ld 的 skernel 和 ekernel 指示。
pub const MEMORY_END: usize = 0x8080_0000;

// 应用程序看到的内存地址空间
// application address space (high)
// |--------------| 2^64
// |  trampoline  | 4 KB
// |--------------|
// | trap context | 4 KB
// |--------------|
// |              |
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

// 注意：
// 关于为什么 TRAMPOLINE, TRAP_CONTEXT 会放在靠近 usize::MAX 的地方，原因：
//
// SV39 分页模式规定 64 位虚拟地址的  这 25 位必须和第 38 位相同，否则 MMU 会直接认定它是一个不合法的虚拟地址。
// 通过这个检查之后 MMU 再取出低 39 位尝试将其转化为一个 56 位的物理地址。
// 也就是说，所有  个虚拟地址中，只有最低的  （当第 38 位为 0 时）以及最高的  （当第 38 位为 1 时）是可能
// 通过 MMU 检查的
//
// http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/3sv39-implementation-1.html#high-and-low-256gib

// 内核看到的内存地址空间
// kernel address space (high)
// |--------------------| 2^64
// |      trampoline    | 4 KB
// |--------------------|
// | app 0 kernel stack | 4 KB
// |         ---        |
// |      guard page    | 4 KB
// |--------------------|
// | app 1 kernel stack | 4 KB
// |         ---        |
// |      guard page    | 4 KB
// |--------------------|
// |                    |
/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

// QEMU 的时钟频率, 12.5MHz
pub const CLOCK_FREQ: usize = 12500000;

// QEMU MMIO
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];