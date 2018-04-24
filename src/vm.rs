use process::Process;
use val::Value;
use std::thread::{ThreadId, JoinHandle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Mutex, Arc, Weak, RwLock};
use io::Fd;
use host_info::cpu_count;


#[derive(Debug)]
struct ProcPool {
    procs: Mutex<HashMap<u64, Ed<Process>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Debug, Clone)]
struct Channel {
    messages: Arc<Mutex<VecDeque<Value>>>,
}

#[derive(Debug, Clone)]
struct Ed<T> {
    dev: T,
    allowed: HashSet<u64>,
}

#[derive(Debug)]
struct VmInner {
    fds: Mutex<HashMap<u64, Ed<Fd>>>,
    ipcs: Mutex<HashMap<u64, Ed<Channel>>>,
    pool: Box<[ProcPool]>,
    fd_inc: Mutex<u64>,
    ipc_inc: Mutex<u64>,
    proc_inc: Mutex<u64>,
    bytecode: RwLock<HashMap<String, Arc<[u8]>>>,
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
        let mut fds = HashMap::with_capacity(64);
        fds.insert(0, Ed { dev: stdin, allowed: HashSet::new() });
        fds.insert(1, Ed { dev: stdout, allowed: HashSet::new() });
        fds.insert(2, Ed { dev: stderr, allowed: HashSet::new() });
        Self {
            inner: Arc::new(VmInner {
                fds: Mutex::new(fds),
                ipcs: Mutex::new(HashMap::new()),
                pool: pool.into_boxed_slice(),
                fd_inc: Mutex::new(3),
                ipc_inc: Mutex::new(0),
                proc_inc: Mutex::new(1),
                bytecode: RwLock::new(HashMap::new()),
            })
        }
    }

    pub fn as_weak(&self) -> WeakVm {
        WeakVm {
            inner: Arc::downgrade(&self.inner),
        }
    }

}
