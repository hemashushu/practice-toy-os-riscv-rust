use crate::{config::CLOCK_FREQ, sbi::set_timer};

use riscv::register::time;

const MICRO_PER_SEC: usize = 1000; // 1 秒 = 1000 毫秒
const TICKS_PER_SEC: usize = 1000; // 设定每 1/1000 秒触发一次中断

/// read the `mtime` register
pub fn get_time() -> usize {
    time::read()
}

/// get current time in milliseconds
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
