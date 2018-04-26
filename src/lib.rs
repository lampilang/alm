#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(unix)]
extern crate libc;

pub mod edi;
pub mod err;
pub mod host_info;
pub mod io;
pub mod ipc;
pub mod process;
pub mod val;
pub mod vm;

trait PrivFrom<T> {
    fn from(val: T) -> Self;
}

trait PrivInto<T> {
    fn into(self) -> T;
}
