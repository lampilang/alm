#[cfg(unix)]
#[path = "unix.rs"]
mod os;

pub use self::os::{
    close,
    open,
    read,
    seek,
    stderr,
    stdin,
    stdout,
    write,
    OsFd,
    RawInput,
    RawOutput,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndOpts {
    I(),
    O(bool),
    IO(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateOpts {
    CreateNew(),
    Create(bool),
    DoNotCreate(bool),
}
