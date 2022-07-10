#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod sbi;

#[macro_use]
mod console;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    // 导入函数（符号），变相获取各个段的地址
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack(); // stack bottom
        fn boot_stack_top(); // stack top
    }

    // 初始化 .bss 段的内存，将该段内存值置为 0
    clear_bss_by_addr(sbss as usize, ebss as usize);

    // 打印 hello world
    println!("{}", "Hello, world!");

    // 打印各个段的起始和终止地址
    println!(
        ".text start: 0x{:x}, .text end: 0x{:x}",
        stext as usize, etext as usize
    );
    println!(
        ".rodata (read-only data) start: 0x{:x}, .rodate (read-only data) end: 0x{:x}",
        srodata as usize, erodata as usize
    );
    println!(
        ".data start: 0x{:x}, .data end: 0x{:x}",
        sdata as usize, edata as usize
    );
    println!(
        ".bss start: 0x{:x}, .bss end: 0x{:x}",
        sbss as usize, ebss as usize
    );
    println!(
        "stack bottom: 0x{:x}, stack top: 0x{:x}",
        boot_stack as usize, boot_stack_top as usize
    );

    panic!("Shutdown machine!");
}

fn clear_bss_by_addr(start_addr: usize, end_addr: usize) {
    (start_addr..end_addr).for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
}
