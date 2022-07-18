/// Get the total number of applications.
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// get applications data
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }

    // 用户应用程序在数据段中的开始/结束地址
    // link_app.S 有如下文本：
    //
    // ```
    // _num_app:
    // .quad M
    // .quad app_0_start
    // .quad app_1_start
    // .quad ...
    // .quad app_N_start
    // .quad app_N_end
    // ```
    //
    // 这里的 _num_app 即数字 `M` 的指针/地址。
    // 上面一共有 M+1 个 int64 数字，共占用 (M+1) * 8 bytes

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };

    assert!(app_id < num_app);

    // load app data
    let src = unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    };

    src
}
