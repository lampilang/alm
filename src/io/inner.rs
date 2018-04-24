use super::raw::{OsFd, close};
use std::sync::Mutex;

#[derive(Debug)]
pub struct FdInner {
    pub os_fd: Option<OsFd>,
    pub in_use: Mutex<bool>,
    pub ibuf: Mutex<Vec<u8>>,
    pub ibuf_size: usize,
    pub obuf: Mutex<Vec<u8>>,
    pub obuf_size: usize,
}

impl FdInner {

    pub fn swap_use_lock(&self, val: bool) -> bool {
        let mut ptr = self.in_use.lock().unwrap();
        let prev = *ptr;
        *ptr = val;
        prev
    }

}

impl Drop for FdInner {

    fn drop(&mut self) {
        close(self.os_fd.take().unwrap());
    }

}
