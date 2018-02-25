use process::Process;
use val::Value;
use std::thread::{ThreadId, JoinHandle};
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, Arc};
use std::fs::File;

#[derive(Debug)]
struct ProcPool {
    procs: Mutex<HashMap<u64, Arc<Process> >>,
    join: Mutex<Option<JoinHandle<()>>>,
    id: ThreadId,
}

#[derive(Debug)]
enum Stream {
    Stdin(),
    Stdout(),
    Stderr(),
    File(File),
}

#[derive(Debug)]
struct Channel {
    messages: VecDeque<Value>,
}

#[derive(Debug)]
pub struct Vm {
    files: Mutex<HashMap<u64, Stream>>,
    channels: Mutex<HashMap<u64, Channel>>,
    file_inc: Mutex<u64>,
    channel_inc: Mutex<u64>,
    proc_inc: Mutex<u64>,
    pool: Box<[ProcPool]>,
}

