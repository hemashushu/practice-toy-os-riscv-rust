use core::arch::global_asm;

use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
    utvec::TrapMode,
};

// use crate::{batch::run_next_app, syscall::syscall};

use crate::{syscall::syscall, task::suspend_current_and_run_next, timer::set_next_trigger};

use self::context::TrapContext;

pub mod context;

global_asm!(include_str!("trap/trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    // 设置 trap
    // trap 类似 "事件监听者"，当 app 调用 ecall 指令时，就会
    // 触发 trap 然后转到 trap 处理器（handler）
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
            // run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
            // run_next_app();
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
    cx
}
