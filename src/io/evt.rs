use super::{
    raw::{
        RawInput,
        RawOutput,
        read as raw_read,
        write as raw_write,
        seek as raw_seek,
    },
    Fd,
    Error,
    SeekFrom,
};
use std::sync::{Arc, MutexGuard};
use std::ops::{DerefMut};
use std::mem;

pub fn read(fd: Fd, amount: usize) -> Input {
    Input {
        fd,
        raw: None,
        status: InputStatus::Pending(amount),
    }
}

pub fn write(fd: Fd, data: Arc<[u8]>) -> Output {
    Output {
        fd,
        raw: None,
        status: OutputStatus::Pending(data),
    }
}

pub fn flush(fd: Fd) -> Flush {
    Flush {
        fd,
        raw: None,
        status: FlushStatus::Pending(),
    }
}

pub fn seek(fd: Fd, from: SeekFrom) -> Seek {
    Seek {
        fd,
        status: SeekStatus::Pending(from),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputStatus {
    Pending(usize),
    Done(Arc<[u8]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputStatus {
    Pending(Arc<[u8]>),
    Done(),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlushStatus {
    Pending(),
    DoneRead(Arc<[u8]>),
    DoneAll(Arc<[u8]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeekStatus {
    Pending(SeekFrom),
    Done(u64),
}

#[derive(Debug)]
pub struct Input {
    fd: Fd,
    raw: Option<RawInput>,
    status: InputStatus,
}

impl Input {

    pub fn is_done(&self) -> bool {
        match self.status {
            InputStatus::Pending(_) => false,
            _ => true,
        }
    }

    pub fn try_fetch(&mut self) -> Result<InputStatus, Error> {
        let amount = match self.status {
            InputStatus::Pending(x) => x,
            ref res => return Ok(res.clone()),
        };

        if let Some(raw) = self.raw.take() {
            return self.try_read(raw, amount);
        }

        let fd = self.fd.inner.os_fd.unwrap();

        let ibuf_size = self.fd.inner.ibuf_size;

        if self.fd.inner.swap_use_lock(true) {
            return Ok(self.status.clone())
        }

        self.status = InputStatus::Done(loop {
            let raw = {
                let mut ibuf = self.fd.inner.ibuf.lock().unwrap();
                if ibuf.len() >= amount {
                    let tmp = ibuf.split_off(amount);
                    break Arc::from(mem::replace(&mut *ibuf, tmp));
                }
                raw_read(fd, amount.max(ibuf_size) - ibuf.len())
            };
            return self.try_read(raw, amount);
        });

        Ok(self.status.clone())
    }

    fn try_read(
        &mut self,
        mut raw: RawInput,
        amount: usize,
    ) -> Result<InputStatus, Error> {
        let done = match raw.try_read() {
            Ok(x) => x,
            Err(e) => {
                self.raw = Some(raw);
                return Err(e);
            }
        };

        if done {
            self.status = InputStatus::Done({
                let mut ibuf = self.fd.inner.ibuf.lock().unwrap();
                let tmp = ibuf.split_off(amount);
                Arc::from(mem::replace(&mut *ibuf, tmp))
            });
            let prev = self.fd.inner.swap_use_lock(false);
            debug_assert!(prev, "Input use lock was badly released");
        } else {
            self.raw = Some(raw);
        }

        Ok(self.status.clone())
    }

}

#[derive(Debug)]
pub struct Output {
    fd: Fd,
    raw: Option<RawOutput>,
    status: OutputStatus,
}

impl Output {

    pub fn is_done(&self) -> bool {
        match self.status {
            OutputStatus::Pending(_) => false,
            _ => true,
        }
    }

    pub fn try_forward(&mut self) -> Result<OutputStatus, Error> {
        if let Some(raw) = self.raw.take() {
            return self.try_write(raw);
        }

        self.raw = {
            let data = match self.status {
                OutputStatus::Pending(ref x) => x,
                ref res => return Ok(res.clone()),
            };

            let fd = self.fd.inner.os_fd.unwrap();

            let obuf_size = self.fd.inner.obuf_size;

            if self.fd.inner.swap_use_lock(true) {
                return Ok(self.status.clone())
            }

            let mut obuf = self.fd.inner.obuf.lock().unwrap();
            obuf.extend_from_slice(&data);

            if obuf_size <= obuf.len() {
                let buf = obuf.split_off(0);
                Some(raw_write(fd, buf))
            } else {
                None
            }
        };

        if let Some(raw) = self.raw.take() {
            return self.try_write(raw);
        }

        self.status = OutputStatus::Done();

        Ok(self.status.clone())
    }

    fn try_write(
        &mut self,
        mut raw: RawOutput,
    ) -> Result<OutputStatus, Error> {
        let done = match raw.try_write() {
            Ok(x) => x,
            Err(e) => {
                self.raw = Some(raw);
                return Err(e);
            }
        };

        if done {
            self.status = OutputStatus::Done();
            let prev = self.fd.inner.swap_use_lock(false);
            debug_assert!(prev, "Output use lock was badly released");
        } else {
            self.raw = Some(raw);
        }

        Ok(self.status.clone())
    }

}

#[derive(Debug)]
pub struct Flush {
    fd: Fd,
    raw: Option<RawOutput>,
    status: FlushStatus,
}


impl Flush {

    pub fn is_done(&self) -> bool {
        match self.status {
            FlushStatus::DoneAll(_) => true,
            _ => false,
        }
    }

    pub fn try_flush(&mut self) -> Result<FlushStatus, Error> {

        match self.status {
            FlushStatus::Pending() => self.status = FlushStatus::DoneRead({
                if self.fd.inner.swap_use_lock(true) {
                    return Ok(self.status.clone());
                }
                let mut ibuf = self.fd.inner.ibuf.lock().unwrap();
                Arc::from(ibuf.split_off(0))
            }),
            FlushStatus::DoneAll(_) => return Ok(self.status.clone()),
            _ => (),
        }

        if let Some(raw) = self.raw.take() {
            return self.try_write(raw);
        }

        let raw = {
            let fd = self.fd.inner.os_fd.unwrap();

            let obuf_size = self.fd.inner.obuf_size;

            let mut obuf = self.fd.inner.obuf.lock().unwrap();

            let buf = obuf.split_off(0);

            raw_write(fd, buf)
        };

        self.try_write(raw)
    }

    fn try_write(
        &mut self,
        mut raw: RawOutput,
    ) -> Result<FlushStatus, Error> {
        let done = match raw.try_write() {
            Ok(x) => x,
            Err(e) => {
                self.raw = Some(raw);
                return Err(e);
            }
        };

        if done {
            let status = mem::replace(&mut self.status, FlushStatus::Pending());
            self.status = match status {
                FlushStatus::DoneRead(r) => FlushStatus::DoneAll(r),
                s => panic!("Invalid status on flush: {:?}", s),
            };
            let prev = self.fd.inner.swap_use_lock(false);
            debug_assert!(prev, "Output use lock was badly released");
        } else {
            self.raw = Some(raw);
        }

        Ok(self.status.clone())
    }

}

#[derive(Debug)]
pub struct Seek {
    fd: Fd,
    status: SeekStatus,
}

impl Seek {

    pub fn is_done(&self) -> bool {
        match self.status {
            SeekStatus::Pending(_) => false,
            _ => true,
        }
    }

    pub fn try_seek(&mut self) -> Result<&SeekStatus, Error> {
        let mode = match self.status {
            SeekStatus::Pending(mode) => mode,
            ref x => return Ok(x),
        };

        if self.fd.inner.swap_use_lock(true) {
            return Ok(&self.status)
        }

        let fd = self.fd.inner.os_fd.unwrap();

        self.status = SeekStatus::Done(raw_seek(fd, mode)?);

        Ok(&self.status)
    }

}
