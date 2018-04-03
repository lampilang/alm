#[cfg(unix)]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

pub use std::io::Error;

#[cfg(unix)]
pub use self::unix::{
    OsFd,
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


pub trait RawEvent {

    type Result;

    fn is_done(&self) -> bool;

    fn try_fetch(&mut self) -> Result<(), Error>;

    fn take(self) -> Self::Result;

}
