use crate::{mm::page_table::translated_byte_buffer, task::current_user_token};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            // let str = core::str::from_utf8(slice).unwrap();
            // print!("{}", str);

            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }

            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
