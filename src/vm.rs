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
pub struct VmOpts {
    stdin: Option<Fd>,
    stdout: Option<Fd>,
    stderr: Option<Fd>,
    cores: Option<usize>,
}

impl VmOpts {

    pub const MIN_CORE_NUM: usize = 2;

    pub fn new() -> Self {
        Self {
            stdin: None,
            stdout: None,
            stderr: None,
            cores: None,
        }
    }

    pub fn stdin(&mut self, fd: Fd) -> &mut Self {
        self.stdin = Some(fd);
        self
    }

    pub fn stdout(&mut self, fd: Fd) -> &mut Self {
        self.stdout = Some(fd);
        self
    }

    pub fn stderr(&mut self, fd: Fd) -> &mut Self {
        self.stderr = Some(fd);
        self
    }

    pub fn cores(&mut self, cores: usize) -> &mut Self {
        if cores >= Self::MIN_CORE_NUM {
            self.cores = Some(cores);
        };
        self
    }

    pub fn alloc(&self) -> VmAlloc {
        let stdin = self.stdin.clone().unwrap_or_else(|| Fd::stdin());
        let stdout = self.stdout.clone().unwrap_or_else(|| Fd::stdout());
        let stderr = self.stderr.clone().unwrap_or_else(|| Fd::stderr());
        let cpus = self.cores.unwrap_or_else(|| match cpu_count() {
            Some(x) if x >= Self::MIN_CORE_NUM => x,
            _ => Self::MIN_CORE_NUM,
        });
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
        VmAlloc {
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

}

#[derive(Debug, Clone)]
pub struct VmAlloc {
    inner: Arc<VmInner>,
}

impl VmAlloc {

    pub fn as_weak(&self) -> WeakVm {
        WeakVm {
            inner: Arc::downgrade(&self.inner),
        }
    }

}
