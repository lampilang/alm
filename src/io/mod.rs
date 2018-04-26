mod evt;
mod inner;
mod raw;

pub use self::{
    evt::{
        Event,
        Flush,
        FlushStatus,
        Input,
        InputStatus,
        Output,
        OutputStatus,
        Seek,
        SeekStatus,
    },
    raw::OsFd,
};
pub use std::io::{Error, SeekFrom};

use self::{
    evt::{flush, read, seek, write},
    inner::FileInner,
    raw::{
        open,
        stderr,
        stdin,
        stdout,
        CreateOpts::{self, *},
        EndOpts::{self, *},
    },
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
    pub fn done(&self) -> Result<File, Error> {
        open(self.path, self.end_opts, self.create_opts)
            .map(|x| unsafe { File::from_raw(x, self.ibuf, self.obuf) })
    }

    pub fn trunc(&mut self) -> &mut Self {
        self.create_opts = DoNotCreate(true);
        self
    }

    pub fn create_or_trunc(&mut self) -> &mut Self {
        self.create_opts = Create(true);
        self
    }

    pub fn do_not_create(&mut self) -> &mut Self {
        self.create_opts = DoNotCreate(false);
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.create_opts = Create(false);
        self
    }

    pub fn create_new(&mut self) -> &mut Self {
        self.create_opts = CreateNew();
        self
    }

    pub fn output_buf_sz(&mut self, size: usize) -> &mut Self {
        self.obuf = size;
        self
    }

    pub fn input_buf_sz(&mut self, size: usize) -> &mut Self {
        self.ibuf = size;
        self
    }

    pub fn read_append(&mut self) -> &mut Self {
        self.end_opts = IO(true);
        self.ibuf = File::DFL_BUF_SZ;
        self.obuf = File::DFL_BUF_SZ;
        self
    }

    pub fn read_write(&mut self) -> &mut Self {
        self.end_opts = IO(false);
        self.ibuf = File::DFL_BUF_SZ;
        self.obuf = File::DFL_BUF_SZ;
        self
    }

    pub fn append(&mut self) -> &mut Self {
        self.end_opts = O(true);
        self.ibuf = 0;
        self.obuf = File::DFL_BUF_SZ;
        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.end_opts = O(false);
        self.ibuf = 0;
        self.obuf = File::DFL_BUF_SZ;
        self
    }

    pub fn read(&mut self) -> &mut Self {
        self.end_opts = I();
        self.ibuf = File::DFL_BUF_SZ;
        self.obuf = 0;
        self
    }

    pub fn new(path: &'a str) -> Self {
        Self {
            path,
            end_opts: I(),
            create_opts: DoNotCreate(/* truncate: */ false),
            ibuf: File::DFL_BUF_SZ,
            obuf: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct File {
    inner: Arc<FileInner>,
}

static mut STDIN: Option<File> = None;
static mut STDOUT: Option<File> = None;
static mut STDERR: Option<File> = None;

static INIT_STDIN: Once = ONCE_INIT;
static INIT_STDOUT: Once = ONCE_INIT;
static INIT_STDERR: Once = ONCE_INIT;

impl File {
    pub const DFL_BUF_SZ: usize = 0x800;

    pub unsafe fn from_raw(
        fd: OsFd,
        ibuf_size: usize,
        obuf_size: usize,
    ) -> Self {
        Self {
            inner: Arc::new(FileInner {
                os_fd: Some(fd),
                in_use: Mutex::new(false),
                ibuf: Mutex::new(Vec::with_capacity(ibuf_size)),
                ibuf_size,
                obuf: Mutex::new(Vec::with_capacity(obuf_size)),
                obuf_size,
            }),
        }
    }

    pub fn stderr() -> Self {
        INIT_STDERR.call_once(|| unsafe {
            STDERR = Some(Self::from_raw(
                stderr().unwrap(),
                0,
                Self::DFL_BUF_SZ,
            ));
        });
        unsafe { STDERR.clone() }.unwrap()
    }

    pub fn stdout() -> Self {
        INIT_STDOUT.call_once(|| unsafe {
            STDOUT = Some(Self::from_raw(
                stdout().unwrap(),
                0,
                Self::DFL_BUF_SZ,
            ));
        });
        unsafe { STDOUT.clone() }.unwrap()
    }

    pub fn stdin() -> Self {
        INIT_STDIN.call_once(|| unsafe {
            STDIN = Some(Self::from_raw(stdin().unwrap(), 0, 0));
        });
        unsafe { STDIN.clone() }.unwrap()
    }

    pub fn seek(&self, mode: SeekFrom) -> Seek { seek(self.clone(), mode) }

    pub fn flush(&self) -> Flush { flush(self.clone()) }

    pub fn write(&self, data: Arc<[u8]>) -> Output { write(self.clone(), data) }

    pub fn read(&self, count: usize) -> Input { read(self.clone(), count) }

    pub fn open<'a>(path: &'a str) -> FdOpener<'a> { FdOpener::new(path) }
}
