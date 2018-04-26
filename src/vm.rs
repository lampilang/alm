use edi::*;
use err::Error;
use host_info::cpu_count;
use io::{Event, File};
use ipc::Channel;
use process::Process;
use std::{
    collections::{
        hash_map::{Entry, HashMap},
        HashSet,
    },
    sync::{Arc, Mutex, RwLock, Weak},
    thread::JoinHandle,
};
use val::Function;

#[derive(Debug)]
struct ProcPool {
    procs: Mutex<HashMap<Pid, Ed<Process>>>,
    evts: Mutex<HashMap<Evd, Ed<Event>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Debug)]
struct VmInner {
    files: Mutex<HashMap<Fd, Ed<File>>>,
    chs: Mutex<HashMap<Chd, Ed<Channel>>>,
    pool: Box<[ProcPool]>,
    fd_inc: Mutex<u64>,
    chd_inc: Mutex<u64>,
    pid_inc: Mutex<u64>,
    bytecode: RwLock<HashMap<String, Arc<[u8]>>>,
}

#[derive(Debug, Clone)]
pub struct WeakVm {
    inner: Weak<VmInner>,
}

impl WeakVm {
    pub fn as_alloc(&self) -> Option<VmAlloc> {
        self.inner.upgrade().map(|inner| {
            VmAlloc {
                inner,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct VmOpts {
    stdin: Option<File>,
    stdout: Option<File>,
    stderr: Option<File>,
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

    pub fn stdin(&mut self, file: File) -> &mut Self {
        self.stdin = Some(file);
        self
    }

    pub fn stdout(&mut self, file: File) -> &mut Self {
        self.stdout = Some(file);
        self
    }

    pub fn stderr(&mut self, file: File) -> &mut Self {
        self.stderr = Some(file);
        self
    }

    pub fn cores(&mut self, cores: usize) -> &mut Self {
        if cores >= Self::MIN_CORE_NUM {
            self.cores = Some(cores);
        };
        self
    }

    pub fn alloc(&self) -> VmAlloc {
        let stdin = self.stdin.clone().unwrap_or_else(|| File::stdin());
        let stdout = self.stdout.clone().unwrap_or_else(|| File::stdout());
        let stderr = self.stderr.clone().unwrap_or_else(|| File::stderr());
        let cpus = self.cores.unwrap_or_else(|| {
            match cpu_count() {
                Some(x) if x >= Self::MIN_CORE_NUM => x,
                _ => Self::MIN_CORE_NUM,
            }
        });
        let mut pool = Vec::with_capacity(cpus);
        for _ in 0..cpus {
            pool.push(ProcPool {
                procs: Mutex::new(HashMap::with_capacity(64)),
                evts: Mutex::new(HashMap::with_capacity(8)),
                join: Mutex::new(None),
            });
        }
        let mut files = HashMap::with_capacity(64);
        files.insert(Fd::STDIN, Ed::new(stdin));
        files.insert(Fd::STDOUT, Ed::new(stdout));
        files.insert(Fd::STDERR, Ed::new(stderr));
        VmAlloc {
            inner: Arc::new(VmInner {
                files: Mutex::new(files),
                chs: Mutex::new(HashMap::new()),
                pool: pool.into_boxed_slice(),
                fd_inc: Mutex::new(3),
                chd_inc: Mutex::new(0),
                pid_inc: Mutex::new(1),
                bytecode: RwLock::new(HashMap::new()),
            }),
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

    pub fn with_pid<F, T>(
        &self,
        accessor: Pid,
        target: Pid,
        mut fun: F,
    ) -> Result<T, Error>
    where
        F: FnMut(&Process) -> T,
    {
        let idx = self.pool_idx(::PrivInto::into(target));
        let procs = self.inner.pool[idx].procs.lock().unwrap();

        let ed = match procs.get(&target) {
            Some(e) => e,
            _ => return Err(Error::NoSuchPid(target)),
        };

        match ed.access(accessor) {
            Some(p) => Ok(fun(p)),
            _ => Err(Error::AccessDenied(Edi::from(target))),
        }
    }

    pub fn allow_on_pid(
        &self,
        accessor: Pid,
        target: Pid,
        new_acc: Pid,
    ) -> Result<(), Error> {
        let idx = self.pool_idx(::PrivInto::into(target));
        let mut procs = self.inner.pool[idx].procs.lock().unwrap();

        let ed = match procs.get(&target) {
            Some(e) => e,
            _ => return Err(Error::NoSuchPid(target)),
        };

        if ed.allow(accessor, new_acc) {
            Ok(())
        } else {
            Err(Error::AccessDenied(Edi::from(target)))
        }
    }

    pub fn allow_on_fd(
        &self,
        accessor: Pid,
        target: Fd,
        new_acc: Pid,
    ) -> Result<(), Error> {
        let mut files = self.inner.files.lock().unwrap();
        let ed = match files.get(&target) {
            Some(e) => e,
            _ => return Err(Error::NoSuchFd(target)),
        };

        if ed.allow(accessor, new_acc) {
            Ok(())
        } else {
            Err(Error::AccessDenied(Edi::from(target)))
        }
    }

    pub fn allow_on_chd(
        &self,
        accessor: Pid,
        target: Chd,
        new_acc: Pid,
    ) -> Result<(), Error> {
        let mut chs = self.inner.chs.lock().unwrap();
        let ed = match chs.get(&target) {
            Some(e) => e,
            _ => return Err(Error::NoSuchChd(target)),
        };

        if ed.allow(accessor, new_acc) {
            Ok(())
        } else {
            Err(Error::AccessDenied(Edi::from(target)))
        }
    }

    pub fn allow_on_evd(
        &self,
        accessor: Pid,
        target: Evd,
        new_acc: Pid,
    ) -> Result<(), Error> {
        let idx = self.pool_idx(::PrivInto::into(target));
        let mut evts = self.inner.pool[idx].evts.lock().unwrap();
        let ed = match evts.get(&target) {
            Some(e) => e,
            _ => return Err(Error::NoSuchEvd(target)),
        };

        if ed.allow(accessor, new_acc) {
            Ok(())
        } else {
            Err(Error::AccessDenied(Edi::from(target)))
        }
    }

    fn pool_idx(&self, id: u64) -> usize {
        (id % self.inner.pool.len() as u64) as usize
    }
}

#[derive(Debug, Clone)]
pub struct Spawner {
    vm: VmAlloc,
    accessor: Pid,
    allow_pids: HashSet<Pid>,
    allow_fds: HashSet<Fd>,
    allow_chds: HashSet<Chd>,
    allow_evds: HashSet<Evd>,
    perms: HashSet<Pid>,
}

impl Spawner {
    pub fn new(vm: VmAlloc, accessor: Pid) -> Self {
        Self {
            vm,
            accessor,
            allow_pids: HashSet::new(),
            allow_fds: HashSet::new(),
            allow_chds: HashSet::new(),
            allow_evds: HashSet::new(),
            perms: HashSet::new(),
        }
    }

    pub fn allow_pid(&mut self, pid: Pid) -> &mut Self {
        self.allow_pids.insert(pid);
        self
    }

    pub fn allow_fd(&mut self, fd: Fd) -> &mut Self {
        self.allow_fds.insert(fd);
        self
    }

    pub fn allow_chd(&mut self, chd: Chd) -> &mut Self {
        self.allow_chds.insert(chd);
        self
    }

    pub fn allow_evd(&mut self, evd: Evd) -> &mut Self {
        self.allow_evds.insert(evd);
        self
    }

    pub fn allow_pids<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Pid>,
    {
        self.allow_pids = iter.into_iter().collect();
        self
    }

    pub fn allow_fds<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Fd>,
    {
        self.allow_fds = iter.into_iter().collect();
        self
    }

    pub fn allow_chds<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Chd>,
    {
        self.allow_chds = iter.into_iter().collect();
        self
    }

    pub fn allow_evds<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Evd>,
    {
        self.allow_evds = iter.into_iter().collect();
        self
    }

    pub fn perm_on_self(&mut self, other: Pid) -> &mut Self {
        self.perms.insert(other);
        self
    }

    pub fn perms_on_self<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Pid>,
    {
        self.perms = iter.into_iter().collect();
        self
    }

    pub fn spawn(&self, starter: Function) -> Result<Pid, Error> {
        let pid = {
            let mut cur = self.vm.inner.pid_inc.lock().unwrap();
            loop {
                let pid = <Pid as ::PrivFrom<_>>::from(*cur);
                let idx = self.vm.pool_idx(*cur);
                let mut procs = self.vm.inner.pool[idx].procs.lock().unwrap();
                *cur = (*cur + 1 & !Edi::MASK).max(1);
                match procs.entry(pid) {
                    Entry::Vacant(entry) => {
                        entry.insert(Ed::from_perms(
                            Process::new(pid, starter, &self.vm),
                            self.perms.iter().map(|&x| x),
                        ));
                        break pid;
                    },
                    _ => (),
                }
            }
        };

        match self.run_permissions(pid) {
            Err(e) => {
                self.vm.with_pid(Pid::KERNEL, pid, Process::cancel).unwrap();
                Err(e)
            },
            _ => {
                self.vm.with_pid(Pid::KERNEL, pid, Process::launch).unwrap();
                Ok(pid)
            },
        }
    }

    fn run_permissions(&self, pid: Pid) -> Result<(), Error> {
        for &sub_pid in &self.allow_pids {
            self.vm.allow_on_pid(self.accessor, sub_pid, pid)?;
        }
        for &fd in &self.allow_fds {
            self.vm.allow_on_fd(self.accessor, fd, pid)?;
        }
        for &chd in &self.allow_chds {
            self.vm.allow_on_chd(self.accessor, chd, pid)?;
        }
        for &evd in &self.allow_evds {
            self.vm.allow_on_evd(self.accessor, evd, pid)?;
        }
        Ok(())
    }
}
