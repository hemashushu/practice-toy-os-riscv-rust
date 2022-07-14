#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod config;
mod lang_items;
mod sbi;
mod up;

// pub mod batch;
mod loader;

pub mod stack;
pub mod syscall;
pub mod task;
pub mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

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
    println!("[kernel] {}", "Hello, world!");

    // 打印各个段的起始和终止地址
    println!(
        ".text start: 0x{:x}, .text end: 0x{:x}",
        stext as usize, etext as usize
    );

    println!(
        ".rodata start: 0x{:x}, .rodata end: 0x{:x}",
        srodata as usize, erodata as usize
    );

    println!(
        ".data start: 0x{:x}, .data end: 0x{:x}",
        sdata as usize, edata as usize
    );

    println!(
        "stack bottom: 0x{:x}, stack top: 0x{:x}",
        boot_stack as usize, boot_stack_top as usize
    );

    println!(
        ".bss start: 0x{:x}, .bss end: 0x{:x}",
        sbss as usize, ebss as usize
    );

    // 最终生成的程序布局
    // 注：静态布局由文件 linker.ld 决定
    //
    // | -------------------------- |
    // | 0x80460000
    // |              // ...        |
    // | 0x80440000:  .text         <-- 应用程序 app 2 被加载到这里
    // | -------------------------- |
    // | 0x80440000
    // |              // ...        |
    // | 0x80420000:  .text         <-- 应用程序 app 1 被加载到这里
    // | -------------------------- |
    // | 0x80420000                 <-- 用户 app 最大允许大小 0x20000 bytes
    // |              // ...        |
    // | 0x80400000:  .text         <-- 应用程序 app 0 被加载到这里
    // | -------------------------- |
    //   high address
    // | -------------------------- |
    // | 0x8022e000:  .bss end      |
    // |              // ...        |
    // | 0x8022d000:  .bss start    |
    // | 0x8022d000:  stack top     |
    // |              // 由 entry.asm 开辟的内核 stack 空间，大小是 64KB
    // | 0x8021d000:  stack bottom  <-- 实际上这里已经是 .bss 段的开始, 内核 stack 在 .bss 的开始端开辟了空间
    // | 0x8021d000:  .data end     |
    // |              // ...        |
    // |              // 第 2 个 app 程序的二进制内容
    // |              // ...
    // | 0x80217028:  // 第 0 个 app 程序的二进制内容（0x80217000 + 5 * 8 = 0x8020b038）
    // |              // 5 个 int64 整数（即数字 3、以及 3 个 app 在 data 段中的开始地址，以及最后一个 app 的结束位置）
    // | 0x80217000:  .data start   |
    // | 0x80217000:  .rodata end   |
    // | 0x80215000:  // USER_STACK 3
    // | 0x80213000:  // USER_STACK 2
    // | 0x80211000:  // USER_STACK 1
    // | 0x8020f000:  // USER_STACK 0   <-- 用户 app 栈（8 KB）
    // | 0x8020d000:  // KERNAL_STACK 3
    // | 0x8020b000:  // KERNAL_STACK 2
    // | 0x80209000:  // KERNAL_STACK 1
    // | 0x80207000:  // KERNAL_STACK 0 <-- 内核 trap 栈（8 KB）
    // |              // ...
    // | 0x80204000:  .rodata start |
    // | 0x80204000:  .text end     |
    // |              // 1. 来自 main.rs 的 函数 rust_main
    // |              // 0. 来自 entry.asm 的 .text.entry
    // | 0x80200000:  .text start   |
    // | -------------------------- |
    //   low address

    trap::init();
    // batch::init();
    // batch::run_next_app();
    loader::load_apps();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

// fn clear_bss() {
//     extern "C" {
//         fn sbss();
//         fn ebss();
//     }
//     let start_addr = sbss as usize;
//     let end_addr = ebss as usize;
//
//     // unsafe {
//     //     core::slice::from_raw_parts_mut(start_addr as *mut u8, end_addr - start_addr).fill(0);
//     // }
//     clear_bss_by_addr(start_addr, end_addr);
// }

fn clear_bss_by_addr(start_addr: usize, end_addr: usize) {
    // (start_addr..end_addr).for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
    unsafe {
        core::slice::from_raw_parts_mut(start_addr as *mut u8, end_addr - start_addr).fill(0);
    }
}
