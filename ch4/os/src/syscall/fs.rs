use crate::{mm::page_table::translated_byte_buffer, task::current_user_token};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // ch4 MODIFY:
            //
            // 因为内核和应用程序的内存空间不一样，这里传过来的 buf 指针是应用程序空间的地址。
            // let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            // let str = core::str::from_utf8(slice).unwrap();
            // print!("{}", str);

            // 不过内核可以访问应用程序的内存空间的任何角落，只需获取应用程序的 root ppn，然后通过查表就能得到
            // 物理地址，然后内核的内存空间是根物理空间一一对应的，所以可以直接访问物理空间的数据
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
