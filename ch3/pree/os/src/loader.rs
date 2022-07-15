use core::arch::asm;

use crate::{
    config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT},
    stack::{KERNEL_STACK, USER_STACK},
    trap::context::TrapContext,
};

/// Get base address of app i.
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

/// Get the total number of applications.
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// Load nth user app at
/// [APP_BASE_ADDRESS + n * APP_SIZE_LIMIT, APP_BASE_ADDRESS + (n+1) * APP_SIZE_LIMIT).
pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }

    // 加载用户应用程序在数据段中的开始/结束地址
    // link_app.S 有如下文本：
    //
    // ```
    // _num_app:
    // .quad 3
    // .quad app_0_start
    // .quad app_1_start
    // .quad app_2_start
    // .quad app_2_end
    // ```
    //
    // 这里的 _num_app 即数字 `3` 的指针/地址。
    // 上面一共有 5 个 int64 数字，共占用 5 * 8 = 40 bytes

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // clear i-cache first
    unsafe {
        asm!("fence.i");
    }
    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // clear region
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });

        println!("[kernel] load user app {} from 0x{:x}", i, app_start[i]);

        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

/// get app info with entry and sp and save `TrapContext` in kernel stack
pub fn init_app_cx(app_id: usize) -> usize {
    println!("[kernel] user app {} stack: 0x{:x}", app_id, USER_STACK[app_id].get_sp());

    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}
