use io::{
    raw::{
        CreateOpts::{self, *},
        EndOpts::{self, *},
    },
    Error,
    SeekFrom,
};
use libc::{
    __errno_location,
    c_int,
    c_void,
    close as c_close,
    fcntl,
    lseek,
    mknod,
    mode_t,
    off_t,
    open as c_open,
    read as c_read,
    write as c_write,
    EOVERFLOW,
    F_GETFL,
    F_SETFL,
    O_APPEND,
    O_CREAT,
    O_NONBLOCK,
    O_RDONLY,
    O_RDWR,
    O_TRUNC,
    O_WRONLY,
    SEEK_CUR,
    SEEK_END,
    SEEK_SET,
    STDERR_FILENO,
    STDIN_FILENO,
    STDOUT_FILENO,
    S_IFREG,
    S_IRGRP,
    S_IROTH,
    S_IRUSR,
    S_IWUSR,
};
use std::io::ErrorKind;

pub const DFL_PERM: mode_t = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

pub type OsFd = c_int;

#[derive(Debug)]
pub struct RawInput {
    fd: OsFd,
    count: usize,
    buf: Vec<u8>,
}

impl RawInput {
    pub fn take(self) -> Vec<u8> { self.buf }

    pub fn buf(&self) -> &[u8] { &self.buf[..] }

    pub fn try_read(&mut self) -> Result<bool, Error> {
        if self.is_done() {
            return Ok(true);
        }

        let res = unsafe {
            let offset = (self.buf.len() - self.count) as isize;
            let ptr = self.buf.as_mut_ptr().offset(offset) as *mut c_void;
            c_read(self.fd, ptr, self.count)
        };

        if res < 0 {
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::WouldBlock {
                return Err(err);
            }
        } else {
            self.count += res as usize;
        }

        Ok(self.is_done())
    }

    pub fn is_done(&self) -> bool { self.count >= self.buf.len() }
}

#[derive(Debug)]
pub struct RawOutput {
    fd: OsFd,
    count: usize,
    buf: Vec<u8>,
}

impl RawOutput {
    pub fn try_write(&mut self) -> Result<bool, Error> {
        if self.is_done() {
            return Ok(true);
        }

        let res = unsafe {
            let offset = self.count as isize;
            let ptr = self.buf.as_ptr().offset(offset) as *const c_void;
            c_write(self.fd, ptr, self.buf.len() - self.count)
        };

        if res < 0 {
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::WouldBlock {
                return Err(err);
            }
        } else {
            self.count += res as usize;
        }

        Ok(self.is_done())
    }

    pub fn is_done(&self) -> bool { self.count >= self.buf.len() }
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
        O(append) => {
            if append {
                O_WRONLY | O_APPEND
            } else {
                O_WRONLY
            }
        },
        IO(append) => {
            if append {
                O_RDWR | O_APPEND
            } else {
                O_RDWR
            }
        },
    };

    let fd = match create {
        CreateNew() => {
            let mode = S_IFREG | DFL_PERM;
            if unsafe { mknod(path_ptr, mode, 0) } < 0 {
                return Err(Error::last_os_error());
            }
            unsafe { c_open(path_ptr, int_flags) }
        },

        Create(trunc) => {
            if trunc {
                int_flags |= O_TRUNC;
            }
            unsafe { c_open(path_ptr, int_flags | O_CREAT, DFL_PERM) }
        },

        DoNotCreate(trunc) => {
            if trunc {
                int_flags |= O_TRUNC;
            }
            unsafe { c_open(path_ptr, int_flags) }
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
        buf: vec![0; count],
        count: 0,
    }
}

pub fn write<'a>(fd: OsFd, buf: Vec<u8>) -> RawOutput {
    RawOutput {
        fd,
        buf,
        count: 0,
    }
}

pub fn seek(fd: OsFd, from: SeekFrom) -> Result<u64, Error> {
    match from {
        SeekFrom::Start(offset) => seek_uint(fd, offset, SEEK_SET),
        SeekFrom::Current(offset) => seek_int(fd, offset, SEEK_CUR),
        SeekFrom::End(offset) => seek_int(fd, offset, SEEK_END),
    }
}

fn seek_uint(fd: OsFd, mut offset: u64, from: c_int) -> Result<u64, Error> {
    while offset > off_t::max_value() as u64 {
        offset -= off_t::max_value() as u64;
        let res = unsafe { lseek(fd, off_t::max_value(), from) };
        if res < 0 && unsafe { *__errno_location() } != EOVERFLOW {
            return Err(Error::last_os_error());
        }
    }

    let res = unsafe { lseek(fd, offset as off_t, from) };
    if res >= 0 {
        Ok(res as u64)
    } else if unsafe { *__errno_location() } == EOVERFLOW {
        Ok(u64::max_value())
    } else {
        Err(Error::last_os_error())
    }
}

fn seek_int(fd: OsFd, mut offset: i64, from: c_int) -> Result<u64, Error> {
    while offset > off_t::max_value() as i64 {
        offset -= off_t::max_value() as i64;
        let res = unsafe { lseek(fd, off_t::max_value(), from) };
        if res < 0 && unsafe { *__errno_location() } != EOVERFLOW {
            return Err(Error::last_os_error());
        }
    }

    while offset < off_t::min_value() as i64 {
        offset -= off_t::min_value() as i64;
        let res = unsafe { lseek(fd, off_t::min_value(), from) };
        if res < 0 && unsafe { *__errno_location() } != EOVERFLOW {
            return Err(Error::last_os_error());
        }
    }

    let res = unsafe { lseek(fd, offset as off_t, from) };
    if res >= 0 {
        Ok(res as u64)
    } else if unsafe { *__errno_location() } == EOVERFLOW {
        Ok(u64::max_value())
    } else {
        Err(Error::last_os_error())
    }
}

pub fn close(fd: OsFd) {
    unsafe {
        c_close(fd);
    }
}

pub fn stdin() -> Result<OsFd, Error> { set_non_blocking(STDIN_FILENO) }

pub fn stdout() -> Result<OsFd, Error> { set_non_blocking(STDOUT_FILENO) }

pub fn stderr() -> Result<OsFd, Error> { set_non_blocking(STDERR_FILENO) }

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
