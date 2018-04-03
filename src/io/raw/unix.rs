use libc::{
    c_void,
    c_int,
    mode_t,
    fcntl,
    open as fd_open,
    close as fd_close,
    mknod,
    read as c_read,
    write as c_write,
    S_IFREG,
    F_GETFL,
    F_SETFL,
    O_NONBLOCK,
    O_APPEND,
    O_CREAT,
    O_TRUNC,
    O_RDONLY,
    O_WRONLY,
    O_RDWR,
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO,
    S_IRUSR,
    S_IWUSR,
    S_IRGRP,
    S_IROTH,
};
use std::io::ErrorKind;
use io::Error;
use io::raw::{EndOpts, CreateOpts, RawEvent};
use io::raw::CreateOpts::*;
use io::raw::EndOpts::*;

pub const DFL_PERM: mode_t = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

pub type OsFd = c_int;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawInput {
    fd: OsFd,
    count: usize,
    read: Vec<u8>,
}

impl RawEvent for RawInput {

    type Result = Vec<u8>;

    fn is_done(&self) -> bool {
        self.count == 0
    }

    fn try_fetch(&mut self) -> Result<(), Error> {
        if self.is_done() {
            return Ok(());
        }
        let res = unsafe {
            let offset = (self.read.len() - self.count) as isize;
            let ptr = self.read.as_mut_ptr().offset(offset) as *mut c_void;
            c_read(self.fd, ptr, self.count)
        };
        if res < 0 {
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::WouldBlock {
                return Err(err);
            }
        } else {
            self.count -= res as usize;
        }
        Ok(())
    }

    fn take(mut self) ->  Self::Result {
        self.read.truncate(self.count);
        self.read
    }

}

impl RawEvent for RawOutput {

    type Result = usize;

    fn is_done(&self) -> bool {
        self.written >= self.data.len()
    }

    fn try_fetch(&mut self) -> Result<(), Error> {
        if self.is_done() {
            return Ok(());
        }
        let res = unsafe {
            let offset = self.written as isize;
            let ptr = self.data.as_ptr().offset(offset) as *const c_void;
            c_write(self.fd, ptr, self.data.len() - self.written)
        };
        if res < 0 {
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::WouldBlock {
                return Err(err);
            }
        } else {
            self.written += res as usize;
        }
        Ok(())
    }

    fn take(self) ->  Self::Result {
        self.written
    }

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawOutput {
    fd: OsFd,
    data: Vec<u8>,
    written: usize,
}

pub fn open(
    path: &str,
    end: EndOpts,
    create: CreateOpts,
) -> Result<OsFd, Error> {
    // please note that we could have used `Vec::from(&str)`, but
    // we would have to push a zero without the proper capacity, and
    // thus reallocating the vector
    let mut path_alloc = Vec::with_capacity(path.len() + 1);
    path_alloc.extend_from_slice(path.as_bytes());
    path_alloc.push(0);
    let path_ptr = path_alloc.as_ptr() as *const i8;
    let mut int_flags = O_NONBLOCK | match end {
        I() => O_RDONLY,
        O(append) => if append {
            O_WRONLY | O_APPEND
        } else {
            O_WRONLY
        },
        IO(append) => if append {
            O_RDWR | O_APPEND
        } else {
            O_RDWR
        },
    };
    let fd = match create {
        CreateNew() => {
            let mode = S_IFREG | DFL_PERM;
            if unsafe { mknod(path_ptr, mode, 0) } < 0 {
                return Err(Error::last_os_error());
            }
            unsafe { fd_open(path_ptr, int_flags) }
        },
        Create(trunc) => {
            if trunc {
                int_flags |= O_TRUNC;
            }
            unsafe { fd_open(path_ptr, int_flags | O_CREAT, DFL_PERM) }
        },
        DoNotCreate(trunc) => {
            if trunc {
                int_flags |= O_TRUNC;
            }
            unsafe { fd_open(path_ptr, int_flags) }
        },
    };
    if fd < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(fd)
    }
}

pub fn read(fd: OsFd, count: usize) -> RawInput {
    RawInput {
        fd,
        count,
        read: vec![0; count],
    }
}

pub fn write(fd: OsFd, data: Vec<u8>) -> RawOutput {
    RawOutput {
        fd,
        data,
        written: 0,
    }
}

pub fn close(fd: OsFd) {
    unsafe { fd_close(fd); }
}

pub fn stdin() -> Result<OsFd, Error> {
    set_non_blocking(STDIN_FILENO)
}

pub fn stdout() -> Result<OsFd, Error> {
    set_non_blocking(STDOUT_FILENO)
}

pub fn stderr() -> Result<OsFd, Error> {
    set_non_blocking(STDERR_FILENO)
}

pub fn set_non_blocking(fd: OsFd) -> Result<OsFd, Error> {
    let flags = unsafe { fcntl(fd, F_GETFL) };
    if flags < 0 {
        return Err(Error::last_os_error());
    }
    if unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) } < 0 {
        return Err(Error::last_os_error());
    }
    Ok(fd)
}
