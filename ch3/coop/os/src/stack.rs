use crate::{
    config::{KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE},
    trap::context::TrapContext,
};

// 内核 trap 栈
// 这个是用于执行内核 trap handler 专门的栈，仅当 app 调用 ecall 触发
// trap 并转为 S 模式时才用到，
// 这个栈跟在 entry.asm 里开辟的 stack 是不同的。
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

// 用户 app 栈
// 用于执行用户应用程序
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

// 内核 trap 栈空间
// `pub static` 的变量被分配到 .bss 段
// 注：经检查实际上被分配到 .rodata 段
pub static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

// 用户 app 栈空间
// `pub static` 的变量被分配到 .bss 段
// 注：经检查实际上被分配到 .rodata 段
pub static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl UserStack {
    // 获取用户 app 栈顶地址
    // 用于切换栈
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    // 获取内核 trap 栈顶地址
    // 用于切换栈
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    // pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        println!("[kernel] push trap context into kernel trap stack: 0x{:x}", self.get_sp());

        // let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}
