#[cfg(unix)]
#[path="unix.rs"]
mod os;

#[cfg(target_os = "windows")]
#[path="windows.rs"]
mod os;

pub use std::io::Error;

pub use self::os::{
    OsFd,
    RawInput,
    RawOutput,
    read,
    write,
    open,
    close,
    stdin,
    stdout,
    stderr,
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
