#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;

mod lang_items;
mod sbi;
mod up;

pub mod batch;
pub mod stack;
pub mod syscall;
pub mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    // 初始化 .bss 段的内存，将该段内存值置为 0
    clear_bss();

    // 打印 hello world
    println!("[kernel] {}", "Hello, world!");

    trap::init();
    batch::init();
    batch::run_next_app();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let start_addr = sbss as usize;
    let end_addr = ebss as usize;

    unsafe {
        core::slice::from_raw_parts_mut(start_addr as *mut u8, end_addr - start_addr).fill(0);
    }
}
