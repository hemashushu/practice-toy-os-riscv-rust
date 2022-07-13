use crate::trap::context::TrapContext;

// 内核 trap 栈和用户栈的大小都是 8KB
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

// 内核 trap 栈
// 这个是用于执行内核 trap handler 专门的栈，仅当 app 调用 ecall 触发
// trap 并转为 S 模式时才用到，
// 这个栈跟在 entry.asm 里开辟的 stack 是不同的。
#[repr(align(4096))]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

// 用户 app 栈
// 用于执行用户应用程序
#[repr(align(4096))]
pub struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

// 内核 trap 栈空间
// `pub static` 的变量被分配到 .bss 段
// 注：经检查实际上被分配到 .rodata 段
pub static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

// 用户 app 栈空间
// `pub static` 的变量被分配到 .bss 段
// 注：经检查实际上被分配到 .rodata 段
pub static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl UserStack {
    // 获取用户栈顶地址
    // 用于切换栈
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        println!("[kernel] push context to: 0x{:x}", cx_ptr as usize);

        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}
