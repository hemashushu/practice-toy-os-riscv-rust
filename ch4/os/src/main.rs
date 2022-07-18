#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
pub mod console;
mod config;
mod lang_items;
mod sbi;
mod timer;
mod up;

// pub mod batch;
mod loader;

pub mod mm;
pub mod syscall;
pub mod task;
pub mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    print_section_info();

    println!("[kernel] Hello, world!");
    mm::init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();

    panic!("Unreachable in rust_main!");
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

fn print_section_info() {
    // 导入函数（符号），变相获取各个段的地址
    extern "C" {
        fn skernel(); // start addr of Kernel
        fn stext(); // begin addr of text segment
        fn strampoline();
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss_with_stack();
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack(); // stack bottom
        fn boot_stack_top(); // stack top
        fn ekernel(); // end addr of Kernel
    }

    // 打印各个段的起始和终止地址

    // 注意：
    // 这里打印的是启用地址空间（/虚拟地址）之前的内存情况，
    // 启用之后的情况请看 `mm/memory_set.rs` 的 `new_kernel()`
    // 对于应用程序所看到的空间，可以查看
    // `mm/memory_set.rs` 的 `from_elf()`
    //
    // `内核地址空间` 和 `应用程序的地址空间` 的图例：
    // http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/5kernel-app-spaces.html#id6
    // http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/5kernel-app-spaces.html#id7

    println!("kernel start: 0x{:x}", skernel as usize);

    println!(
        ".text start: 0x{:x}, .text end: 0x{:x}",
        stext as usize, etext as usize
    );

    println!("strampoline: 0x{:x}", strampoline as usize);

    println!(
        ".rodata start: 0x{:x}, .rodata end: 0x{:x}",
        srodata as usize, erodata as usize
    );

    println!(
        ".data start: 0x{:x}, .data end: 0x{:x}",
        sdata as usize, edata as usize
    );

    println!("sbss_with_stack: 0x{:x}", sbss_with_stack as usize);

    println!(
        "stack bottom: 0x{:x}, stack top: 0x{:x}",
        boot_stack as usize, boot_stack_top as usize
    );

    println!(
        ".bss start: 0x{:x}, .bss end: 0x{:x}",
        sbss as usize, ebss as usize
    );

    println!("kernel end: 0x{:x}", ekernel as usize);
}
