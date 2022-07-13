use core::arch::asm;

use lazy_static::lazy_static;

use crate::{
    stack::{KERNEL_STACK, KERNEL_STACK_SIZE, USER_STACK, USER_STACK_SIZE},
    trap::context::TrapContext,
    up::UPSafeCell,
};

const MAX_APP_NUM: usize = 16;

struct AppManager {
    num_app: usize,                      // app 的总数量
    current_app: usize,                  // 当前 app 的索引值
    app_start: [usize; MAX_APP_NUM + 1], // 每个 app 的数据的起始位置
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }

            // 加载用户应用程序在数据段中的开始/结束地址
            // link_app.S 有如下文本：
            //
            // ```
            // _num_app:
            // .quad 5
            // .quad app_0_start
            // .quad app_1_start
            // .quad app_2_start
            // .quad app_3_start
            // .quad app_4_start
            // .quad app_4_end
            // ```
            //
            // 这里的 _num_app 即数字 `5` 的指针/地址。
            // 上面一共有 7 个 int64 数字，共占用 7 * 8 = 56 bytes
            let num_app_ptr = _num_app as usize as *const usize;

            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }

        println!(
            "[kernel] kernel stack: 0x{:x} - 0x{:x}",
            KERNEL_STACK.get_sp(),
            KERNEL_STACK.get_sp() + KERNEL_STACK_SIZE
        );

        println!(
            "[kernel] user stack: 0x{:x} - 0x{:x}",
            USER_STACK.get_sp(),
            USER_STACK.get_sp() + USER_STACK_SIZE
        );
    }

    // 从内核“程序”里的数据段里加载用户应用程序的指令到指定位置
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("[kernel] All applications completed!");
        }

        println!("[kernel] Loading app_{}", app_id);

        // 清除指令缓存
        // clear icache
        asm!("fence.i");

        // clear app area
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// run next app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }

    // 加载完当前 app 之后，递增 self.current_app 的值，让下一次
    // run_next_app() 时加载下一个 app。
    // P.S.
    // 函数 run_next_app() 表达的是 "run the current app and get ready for the next"
    app_manager.move_to_next_app();

    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources

    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}
