use super::{
    raw::{RawInput, RawOutput},
    Fd,
    Error,
};

pub fn read<'a>(fd: Fd, amount: usize) -> Input<'a> {
    Input {
        fd,
        raw: None,
        read: vec![0; amount],
        done: false,
    }
}

pub fn write<'a>(fd: Fd, data: Vec<u8>) -> Output<'a> {
    Output {
        fd,
        raw: None,
        data: Some(data),
        done: false,
    }
}

pub fn flush<'a>(fd: Fd) -> Output<'a> {
    Output {
        fd,
        raw: None,
        data: None,
        done: false,
    }
}

#[derive(Debug)]
pub struct Input<'a> {
    fd: Fd,
    raw: Option<RawInput<'a>>,
    read: Vec<u8>,
    done: bool,
}

#[derive(Debug)]
pub struct Output<'a> {
    fd: Fd,
    raw: Option<RawOutput<'a>>,
    data: Option<Vec<u8>>,
    done: bool,
}
