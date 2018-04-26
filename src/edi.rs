use std::{
    collections::HashSet,
    fmt,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Match<P, F, C, E> {
    pub pid: P,
    pub fd: F,
    pub chd: C,
    pub evd: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edi(u64);

impl Edi {
    pub const EVD: u64 = 0x3 << 62;
    pub const CHD: u64 = 0x2 << 62;
    pub const FD: u64 = 0x1 << 62;
    pub const PID: u64 = 0x0 << 62;
    pub const MASK: u64 = 0x3 << 62;

    pub fn kind<P, F, C, E, T>(self, mut mat: Match<P, F, C, E>) -> T
    where
        P: FnMut(Pid) -> T,
        F: FnMut(Fd) -> T,
        C: FnMut(Chd) -> T,
        E: FnMut(Evd) -> T,
    {
        match self.0 & Self::MASK {
            Self::PID => (mat.pid)(Pid(self.0 & !Self::MASK)),
            Self::FD => (mat.fd)(Fd(self.0 & !Self::MASK)),
            Self::CHD => (mat.chd)(Chd(self.0 & !Self::MASK)),
            Self::EVD => (mat.evd)(Evd(self.0 & !Self::MASK)),
            _ => panic!("Irrefutable match on effect device id failed!"),
        }
    }
}

impl From<Pid> for Edi {
    fn from(id: Pid) -> Self { Edi(id.0 | Self::PID) }
}

impl Into<Option<Pid>> for Edi {
    fn into(self) -> Option<Pid> {
        if self.0 & Self::MASK == Self::PID {
            Some(Pid(self.0 & !Self::MASK))
        } else {
            None
        }
    }
}

impl From<Fd> for Edi {
    fn from(desc: Fd) -> Self { Edi(desc.0 | Self::FD) }
}

impl Into<Option<Fd>> for Edi {
    fn into(self) -> Option<Fd> {
        if self.0 & Self::MASK == Self::FD {
            Some(Fd(self.0 & !Self::MASK))
        } else {
            None
        }
    }
}

impl From<Chd> for Edi {
    fn from(desc: Chd) -> Self { Edi(desc.0 | Self::CHD) }
}

impl Into<Option<Chd>> for Edi {
    fn into(self) -> Option<Chd> {
        if self.0 & Self::MASK == Self::CHD {
            Some(Chd(self.0 & !Self::MASK))
        } else {
            None
        }
    }
}

impl From<Evd> for Edi {
    fn from(desc: Evd) -> Self { Edi(desc.0 | Self::EVD) }
}

impl Into<Option<Evd>> for Edi {
    fn into(self) -> Option<Evd> {
        if self.0 & Self::MASK == Self::EVD {
            Some(Evd(self.0 & !Self::MASK))
        } else {
            None
        }
    }
}

impl fmt::Display for Edi {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fn show<T: fmt::Display>(val: T) -> String { format!("{}", val) }
        write!(
            fmtr,
            "effect {}",
            self.kind(Match {
                pid: show,
                fd: show,
                chd: show,
                evd: show,
            })
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pid(u64);

impl Pid {
    pub const KERNEL: Self = Pid(0);
}

impl ::PrivFrom<u64> for Pid {
    fn from(val: u64) -> Self {
        debug_assert_eq!(val & Edi::MASK, 0);
        Pid(val)
    }
}

impl ::PrivInto<u64> for Pid {
    fn into(self) -> u64 { self.0 }
}

impl fmt::Display for Pid {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "pid@{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fd(u64);

impl Fd {
    pub const STDERR: Self = Fd(2);
    pub const STDOUT: Self = Fd(1);
    pub const STDIN: Self = Fd(0);
}

impl ::PrivFrom<u64> for Fd {
    fn from(val: u64) -> Self {
        debug_assert_eq!(val & Edi::MASK, 0);
        Fd(val)
    }
}

impl ::PrivInto<u64> for Fd {
    fn into(self) -> u64 { self.0 }
}

impl fmt::Display for Fd {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "file@{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Chd(u64);

impl ::PrivFrom<u64> for Chd {
    fn from(val: u64) -> Self {
        debug_assert_eq!(val & Edi::MASK, 0);
        Chd(val)
    }
}

impl ::PrivInto<u64> for Chd {
    fn into(self) -> u64 { self.0 }
}

impl fmt::Display for Chd {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "channel@{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Evd(u64);

impl ::PrivFrom<u64> for Evd {
    fn from(val: u64) -> Self {
        debug_assert_eq!(val & Edi::MASK, 0);
        Evd(val)
    }
}

impl ::PrivInto<u64> for Evd {
    fn into(self) -> u64 { self.0 }
}

impl fmt::Display for Evd {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "event@{}", self.0)
    }
}

#[derive(Debug)]
struct EdInner<T> {
    dev: T,
    perms: RwLock<HashSet<Pid>>,
}

#[derive(Debug, Clone)]
pub struct Ed<T> {
    inner: Arc<EdInner<T>>,
}

impl<T> Ed<T> {
    pub fn access(&self, accessor: Pid) -> Option<&T> {
        if self.is_allowed(accessor) {
            Some(&self.inner.dev)
        } else {
            None
        }
    }

    pub fn allow(&self, accessor: Pid, new_acc: Pid) -> bool {
        let allowed = self.is_allowed(accessor);
        if allowed {
            self.inner
                .perms
                .write()
                .unwrap()
                .insert(new_acc);
        }
        allowed
    }

    pub fn is_allowed(&self, accessor: Pid) -> bool {
        accessor == Pid::KERNEL
            || self.inner
                .perms
                .read()
                .unwrap()
                .contains(&accessor)
    }

    pub fn from_perms<I>(dev: T, perms: I) -> Self
    where
        I: IntoIterator<Item = Pid>,
    {
        Self {
            inner: Arc::new(EdInner {
                dev,
                perms: RwLock::new(perms.into_iter().collect()),
            }),
        }
    }

    pub fn new(dev: T) -> Self {
        Self {
            inner: Arc::new(EdInner {
                dev,
                perms: RwLock::new(HashSet::new()),
            }),
        }
    }
}
