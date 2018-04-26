use edi::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    AccessDenied(Edi),
    NoSuchPid(Pid),
    NoSuchFd(Fd),
    NoSuchChd(Chd),
    NoSuchEvd(Evd),
}
