extern crate alm;

use alm::ast;
use alm::{RegSet, Vm};

fn main() {
    let ops = [
        ast::Op::Add(ast::WordReg::Wa, ast::WordReg::Wb),
        ast::Op::Neg(ast::WordReg::Wc),
    ];
    let regs = RegSet {
        wa: 5,
        wb: 8,
        wc: 10,
        wd: 0,
    };
    let mut vm = Vm::new(regs, ast::compile(&ops));
    println!("{:?}", vm.regs());
    vm.run();
    println!("{:?}", vm.regs());
}
