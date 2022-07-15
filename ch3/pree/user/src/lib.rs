#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;

mod lang_items;
mod syscall;

use syscall::{sys_exit, sys_get_time, sys_write, sys_yield};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // 初始化 .bss 段的内存，将该段内存值置为 0
    clear_bss();

    let exit_code = main();
    exit(exit_code);
    panic!("unreachable after sys_exit!");
}

/// 后备的 main() 函数，用于防止 bin 里面的程序缺少了 main() 函数。
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("can not find the \"main\" function");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    let start_addr = start_bss as usize;
    let end_addr = end_bss as usize;
    (start_addr..end_addr).for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}
