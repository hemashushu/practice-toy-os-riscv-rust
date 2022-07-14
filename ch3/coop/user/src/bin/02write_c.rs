#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::yield_;

const WIDTH: usize = 10;
const HEIGHT: usize = 3;

#[no_mangle]
fn main() -> i32 {
    // 导入函数（符号），变相获取各个段的地址
    extern "C" {
        fn text(); // begin addr of text segment
        fn rodata(); // start addr of Read-Only data segment
        fn data(); // start addr of data segment
        fn start_bss(); // start addr of BSS segment
        fn end_bss(); // end addr of BSS segment
    }

    println!("App c");
    println!("----------");
    // 打印各个段的起始和终止地址
    println!(".text start: 0x{:x}", text as usize);
    println!(".rodata start: 0x{:x}", rodata as usize);
    println!(".data start: 0x{:x}", data as usize);
    println!(".bss start: 0x{:x}", start_bss as usize);
    println!(".bss end: 0x{:x}", end_bss as usize);
    println!("----------");

    for i in 0..HEIGHT {
        for _ in 0..WIDTH {
            print!("C");
        }
        println!(" [{}/{}]", i + 1, HEIGHT);
        yield_();
    }
    println!("Test write_c OK!");
    0
}
