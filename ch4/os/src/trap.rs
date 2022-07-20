use core::arch::global_asm;

use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
    utvec::TrapMode,
};

// use crate::{batch::run_next_app, syscall::syscall};

use crate::{
    config::{TRAMPOLINE, TRAP_CONTEXT},
    syscall::syscall,
    task::{current_trap_cx, current_user_token, suspend_current_and_run_next, exit_current_and_run_next},
    timer::set_next_trigger,
};

pub mod context;

global_asm!(include_str!("trap/trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    // 注：我们把 stvec 设置为内核和应用地址空间共享的跳板页面的
    // 起始地址 TRAMPOLINE 而不是编译器在链接时看到的 __alltraps 的地址。
    // 这是因为启用分页模式之后，内核只能通过跳板页面上的虚拟地址来//
    // 实际取得 __alltraps 和 __restore 的汇编代码。
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

#[no_mangle]
pub fn trap_handler() -> ! {
    // cx: &mut TrapContext) -> &mut TrapContext {
    // ch4 新增 --- \
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    // ch4 新增 --- /

    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // cx
    trap_return();
}

use core::arch::asm;

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    extern "C" {
        fn __alltraps();
        fn __restore();
    }

    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;

    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",                  // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,    // .
            in("a0") trap_cx_ptr,               // a0 = virt addr of Trap Context
            in("a1") user_satp,                 // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}
