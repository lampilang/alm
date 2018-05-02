use dynasmrt::{ExecutableBuffer, DynasmApi};
use dynasmrt::x64::Assembler;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WordReg {
    Wa,
    Wb,
    Wc,
    Wd,
}

impl WordReg {
    pub fn to_dynreg(self) -> u8 {
        match self {
            WordReg::Wa => 0,
            WordReg::Wb => 1,
            WordReg::Wc => 2,
            WordReg::Wd => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    Add(WordReg, WordReg),
    Neg(WordReg),
    Mov(WordReg, WordReg),
    ImmMov(WordReg, usize),
}


pub fn compile(ast: &[Op]) -> ExecutableBuffer {
    let mut asm = Assembler::new().unwrap();
    dynasm!(asm
        ; mov rax, [rdi]
        ; mov rcx, [rdi + 8]
        ; mov rdx, [rdi + 16]
        ; mov rsi, [rdi + 24]
    );
    for op in ast {
        match op {
            &Op::Add(dest, src) => dynasm!(asm
                ; add Rq(dest.to_dynreg()), Rq(src.to_dynreg())
            ),
            &Op::Neg(dest) => dynasm!(asm
                ; neg Rq(dest.to_dynreg())
            ),
            &Op::Mov(dest, src) => dynasm!(asm
                ; mov Rq(dest.to_dynreg()), Rq(src.to_dynreg())
            ),
            &Op::ImmMov(dest, src) => dynasm!(asm
                ; mov Rq(dest.to_dynreg()), QWORD src as u64 as i64
            ),
        }
    }
    dynasm!(asm
        ; mov [rdi], rax
        ; mov [rdi + 8], rcx
        ; mov [rdi + 16], rdx
        ; mov [rdi + 24], rsi
        ; ret
    );
    asm.finalize().unwrap()
}
