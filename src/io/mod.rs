mod raw;
mod inner;
mod evt;

pub use self:: {
    evt::{
        Input,
        Output,
        Flush,
        Seek,
        InputStatus,
        OutputStatus,
        FlushStatus,
        SeekStatus,
    },
    raw::OsFd,
};
pub use std::io::{Error, SeekFrom};

use self::{
    raw::{
        EndOpts,
        CreateOpts,
        EndOpts::*,
        CreateOpts::*,
        open,
        stdin,
        stdout,
        stderr,
    },
    evt::{
        read,
        write,
        flush,
        seek,
    },
    inner::FdInner,
};
use std::sync::{Arc, Mutex, Once, ONCE_INIT};

#[derive(Clone, Debug)]
pub struct FdOpener<'a> {
    path: &'a str,
    end_opts: EndOpts,
    create_opts: CreateOpts,
    ibuf: usize,
    obuf: usize,
}

impl<'a> FdOpener<'a> {

    pub fn new(path: &'a str) -> Self {
        Self {
            path,
            end_opts: I(),
            create_opts: DoNotCreate(/* truncate: */ false),
            ibuf: Fd::DFL_BUF_SZ,
            obuf: 0,
        }
    }

    pub fn read(&mut self) -> &mut Self {
        self.end_opts = I();
        self.ibuf = Fd::DFL_BUF_SZ;
        self.obuf = 0;
        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.end_opts = O(false);
        self.ibuf = 0;
        self.obuf = Fd::DFL_BUF_SZ;
        self
    }

    pub fn append(&mut self) -> &mut Self {
        self.end_opts = O(true);
        self.ibuf = 0;
        self.obuf = Fd::DFL_BUF_SZ;
        self
    }

    pub fn read_write(&mut self) -> &mut Self {
        self.end_opts = IO(false);
        self.ibuf = Fd::DFL_BUF_SZ;
        self.obuf = Fd::DFL_BUF_SZ;
        self
    }

    pub fn read_append(&mut self) -> &mut Self {
        self.end_opts = IO(true);
        self.ibuf = Fd::DFL_BUF_SZ;
        self.obuf = Fd::DFL_BUF_SZ;
        self
    }

    pub fn input_buf_sz(&mut self, size: usize) -> &mut Self {
        self.ibuf = size;
        self
    }

    pub fn output_buf_sz(&mut self, size: usize) -> &mut Self {
        self.obuf = size;
        self
    }

    pub fn create_new(&mut self) -> &mut Self {
        self.create_opts = CreateNew();
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.create_opts = Create(false);
        self
    }

    pub fn do_not_create(&mut self) -> &mut Self {
        self.create_opts = DoNotCreate(false);
        self
    }

    pub fn create_or_trunc(&mut self) -> &mut Self {
        self.create_opts = Create(true);
        self
    }

    pub fn trunc(&mut self) -> &mut Self {
        self.create_opts = DoNotCreate(true);
        self
    }

    pub fn done(&self) -> Result<Fd, Error> {
        open(self.path, self.end_opts, self.create_opts).map(|x| unsafe {
            Fd::from_raw(x, self.ibuf, self.obuf)
        })
    }

}

#[derive(Clone, Debug)]
pub struct Fd {
    inner: Arc<FdInner>,
}

static mut STDIN: Option<Fd> = None;
static mut STDOUT: Option<Fd> = None;
static mut STDERR: Option<Fd> = None;

static INIT_STDIN: Once = ONCE_INIT;
static INIT_STDOUT: Once = ONCE_INIT;
static INIT_STDERR: Once = ONCE_INIT;

impl Fd {

    pub const DFL_BUF_SZ: usize = 0x800;

    pub fn open<'a>(path: &'a str) -> FdOpener<'a> {
        FdOpener::new(path)
    }

    pub fn read(&self, count: usize) -> Input {
        read(self.clone(), count)
    }

    pub fn write(&self, data: Arc<[u8]>) -> Output {
        write(self.clone(), data)
    }

    pub fn flush(&self) -> Flush {
        flush(self.clone())
    }

    pub fn seek(&self, mode: SeekFrom) -> Seek {
        seek(self.clone(), mode)
    }

    pub fn stdin() -> Self {
        INIT_STDIN.call_once(|| unsafe {
            STDIN = Some(Self::from_raw(stdin().unwrap(), 0, 0));
        });
        unsafe { STDIN.clone() }.unwrap()
    }

    pub fn stdout() -> Self {
        INIT_STDOUT.call_once(|| unsafe {
            STDOUT = Some(
                Self::from_raw(stdout().unwrap(), 0, Self::DFL_BUF_SZ)
            );
        });
        unsafe { STDOUT.clone() }.unwrap()
    }

    pub fn stderr() -> Self {
        INIT_STDERR.call_once(|| unsafe {
            STDERR = Some(
                Self::from_raw(stderr().unwrap(), 0, Self::DFL_BUF_SZ)
            );
        });
        unsafe { STDERR.clone() }.unwrap()
    }

    pub unsafe fn from_raw(
        fd: OsFd,
        ibuf_size: usize,
        obuf_size: usize,
    ) -> Self {
        Self {
            inner: Arc::new(FdInner {
                os_fd: Some(fd),
                in_use: Mutex::new(false),
                ibuf: Mutex::new(Vec::with_capacity(ibuf_size)),
                ibuf_size,
                obuf: Mutex::new(Vec::with_capacity(obuf_size)),
                obuf_size,
            })
        }
    }

}
