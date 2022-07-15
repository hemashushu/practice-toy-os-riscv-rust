#![no_std]
#![no_main]

#[macro_use]
extern crate user;

const LEN: usize = 100;

#[no_mangle]
fn main() -> i32 {
    let p = 3u64;
    let m = 998244353u64;
    let iter: usize = 2000000; // >> 为了便于观察结果，这里比原教程的数值大了 10 倍
    let mut s = [0u64; LEN];
    let mut cur = 0usize;
    s[cur] = 1;
    for i in 1..=iter {
        let next = if cur + 1 == LEN { 0 } else { cur + 1 };
        s[next] = s[cur] * p % m;
        cur = next;
        if i % 100000 == 0 { // >> 为了便于观察结果，这里比原教程的数值大了 10 倍
            println!("power_3 [{}/{}]", i, iter);
        }
    }
    println!("{}^{} = {}(MOD {})", p, iter, s[cur], m);
    println!("Test power_3 OK!");
    0
}
