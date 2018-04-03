use val::*;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use vm::{VmAlloc, WeakVm};

#[derive(Clone, Debug)]
pub struct RegSet {
    pub va: Value,
    pub vb: Value,
    pub vc: Value,
    pub vd: Value,
    pub ba: u8,
    pub bb: u8,
    pub bc: u8,
    pub bd: u8,
    pub iwa: isize,
    pub iwb: isize,
    pub iwc: isize,
    pub iwd: isize,
    pub uwa: usize,
    pub uwb: usize,
    pub uwc: usize,
    pub uwd: usize,
    pub ia: i64,
    pub ib: i64,
    pub ic: i64,
    pub id: i64,
    pub ua: u64,
    pub ub: u64,
    pub uc: u64,
    pub ud: u64,
    pub fa: f64,
    pub fb: f64,
    pub fc: f64,
    pub fd: f64,
    pub aa: Array,
    pub ab: Array,
    pub ac: Array,
    pub ad: Array,
    pub ta: Tuple,
    pub tb: Tuple,
    pub tc: Tuple,
    pub td: Tuple,
    pub fna: Function,
    pub fnb: Function,
    pub fnc: Function,
    pub fnd: Function,
    pub sa: Arc<[u8]>,
    pub sb: Arc<[u8]>,
    pub sc: Arc<[u8]>,
    pub sd: Arc<[u8]>,
    rfn: Function,
    ip: usize,
}

#[derive(Debug)]
pub struct Process {
    regs: Mutex<RegSet>,
    fd_wl: Mutex<HashSet<u64>>,
    pid: u64,
    vm: WeakVm,
}

impl Process {

    pub fn new(
        start: Function,
        fds: HashSet<u64>,
        pid: u64,
        vm: &VmAlloc
    ) -> Self {
        Self {
            fd_wl: Mutex::new(fds),
            pid,
            vm: vm.as_weak(),
            regs: Mutex::new(RegSet {
                va: Value::Nil(),
                vb: Value::Nil(),
                vc: Value::Nil(),
                vd: Value::Nil(),
                ba: 0,
                bb: 0,
                bc: 0,
                bd: 0,
                iwa: 0,
                iwb: 0,
                iwc: 0,
                iwd: 0,
                uwa: 0,
                uwb: 0,
                uwc: 0,
                uwd: 0,
                ia: 0,
                ib: 0,
                ic: 0,
                id: 0,
                ua: 0,
                ub: 0,
                uc: 0,
                ud: 0,
                fa: 0.0,
                fb: 0.0,
                fc: 0.0,
                fd: 0.0,
                aa: Array::new(Arc::new(ArrayType::Nil(0))),
                ab: Array::new(Arc::new(ArrayType::Nil(0))),
                ac: Array::new(Arc::new(ArrayType::Nil(0))),
                ad: Array::new(Arc::new(ArrayType::Nil(0))),
                ta: Tuple::new(Arc::new([])),
                tb: Tuple::new(Arc::new([])),
                tc: Tuple::new(Arc::new([])),
                td: Tuple::new(Arc::new([])),
                fna: Function::new(
                    Value::Nil(),
                    Arc::new([]),
                    Arc::from(b"@<no op>".to_vec())
                ),
                fnb: Function::new(
                    Value::Nil(),
                    Arc::new([]),
                    Arc::from(b"@<no op>".to_vec())
                ),
                fnc: Function::new(
                    Value::Nil(),
                    Arc::new([]),
                    Arc::from(b"@<no op>".to_vec())
                ),
                fnd: Function::new(
                    Value::Nil(),
                    Arc::new([]),
                    Arc::from(b"@<no op>".to_vec())
                ),
                sa: Arc::new([]),
                sb: Arc::new([]),
                sc: Arc::new([]),
                sd: Arc::new([]),
                rfn: start,
                ip: 0,
            })
        }
    }

}
