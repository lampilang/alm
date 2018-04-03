use process::Process;
use val::Value;
use std::thread::{ThreadId, JoinHandle};
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, Arc, Weak};
use io::Fd;
use host_info::cpu_count;


#[derive(Debug)]
struct ProcPool {
    procs: Mutex<HashMap<u64, Arc<Process>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Debug)]
struct Channel {
    messages: VecDeque<Value>,
}

#[derive(Debug)]
enum Ed {
    Pid(u64),
    Ipc(Channel),
    Fd(Fd),
}

#[derive(Debug)]
struct VmInner {
    eds: Mutex<HashMap<u64, Ed>>,
    ed_inc: Mutex<u64>,
    pool: Box<[ProcPool]>,
}

#[derive(Debug, Clone)]
pub struct WeakVm {
    inner: Weak<VmInner>,
}

impl WeakVm {

    pub fn as_alloc(&self) -> Option<VmAlloc> {
        self.inner.upgrade().map(|inner| VmAlloc {inner})
    }

}

#[derive(Debug, Clone)]
pub struct VmAlloc {
    inner: Arc<VmInner>,
}

impl VmAlloc {

    pub const MIN_CORE_NUM: usize = 2;

    pub fn new() -> Self {
        Self::piped(Fd::stdin(), Fd::stdout(), Fd::stderr())
    }

    pub fn piped(stdin: Fd, stdout: Fd, stderr: Fd) -> Self {
        let cpus = match cpu_count() {
            Some(x) if x >= Self::MIN_CORE_NUM => x,
            _ => Self::MIN_CORE_NUM,
        };
        let mut pool = Vec::with_capacity(cpus);
        for _ in 0..cpus {
            pool.push(ProcPool {
                procs: Mutex::new(HashMap::with_capacity(64)),
                join: Mutex::new(None),
            });
        }
        let mut eds = HashMap::with_capacity(64);
        eds.insert(0, Ed::Fd(stdin));
        eds.insert(1, Ed::Fd(stdout));
        eds.insert(2, Ed::Fd(stderr));
        Self {
            inner: Arc::new(VmInner {
                eds: Mutex::new(eds),
                ed_inc: Mutex::new(3),
                pool: pool.into_boxed_slice()
            })
        }
    }

    pub fn as_weak(&self) -> WeakVm {
        WeakVm {
            inner: Arc::downgrade(&self.inner),
        }
    }

}
