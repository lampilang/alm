#![feature(plugin)]
#![plugin(dynasm)]

extern crate dynasmrt;

use dynasmrt::ExecutableBuffer;

pub mod ast;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RegSet {
    pub wa: usize,
    pub wb: usize,
    pub wc: usize,
    pub wd: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct Vm {
    regs: RegSet,
    code: ExecutableBuffer,
}

impl Vm {

    pub fn new(regs: RegSet, code: ExecutableBuffer) -> Self {
        Self {regs, code}
    }

    pub fn regs(&self) -> &RegSet {
        &self.regs
    }

    pub fn regs_mut(&mut self) -> &mut RegSet {
        &mut self.regs
    }

    pub fn run(&mut self) {
        let fun = unsafe {
            let ptr = (&self.code.as_ptr()) as *const *const u8;
            *(ptr as *const extern "C" fn(*mut RegSet))
        };
        fun(&mut self.regs as *mut _);
    }

}
