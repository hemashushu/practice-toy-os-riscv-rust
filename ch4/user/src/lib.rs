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
    print_section_info();

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

// 打印各个段的起始和终止地址
fn print_section_info() {
    extern "C" {
        fn text(); // begin addr of text segment
        fn rodata(); // start addr of Read-Only data segment
        fn data(); // start addr of data segment
        fn bss(); // start addr of BSS segment
        fn end(); // end addr of BSS segment
    }

    println!("----------");
    println!(".text:   0x{:x}", text as usize);
    println!(".rodata: 0x{:x}", rodata as usize);
    println!(".data:   0x{:x}", data as usize);
    println!(".bss:    0x{:x}", bss as usize);
    println!("end:     0x{:x}", end as usize);
    println!("----------");
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
