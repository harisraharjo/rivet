use std::mem::Discriminant;

use super::Register;

// #[derive(Debug)]
// pub enum Opcode {
//     /// HALT
//     HLT,
//     /// Illegal
//     IGL,
//     NOP,
//     LOAD,
//     ADD,
//     SUB,
//     MUL,
//     DIV,
// }

#[derive(Debug)]
pub enum Operand {
    Register(Register),
    Immediate(i32),
    Address(u32),
    LabelRef(u32), // For jump/call targets
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Load {
        dest: Register,
        src: Operand,
    },
    Store {
        src: Register,
        dest: Operand,
    },
    Move {
        dest: Register,
        src: Operand,
    },
    Push {
        src: Operand,
    },
    Pop {
        dest: Register,
    },
    Add {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Sub {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Mul {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Div {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    And {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Or {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Xor {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    Shl {
        dest: Register,
        src: Operand,
        shift: Operand,
    },
    Shr {
        dest: Register,
        src: Operand,
        shift: Operand,
    },
    Cmp {
        left: Operand,
        right: Operand,
    },
    Jmp {
        target: Operand,
    },
    Je {
        target: Operand,
    },
    Jne {
        target: Operand,
    },
    Jg {
        target: Operand,
    },
    Jl {
        target: Operand,
    },
    Call {
        target: Operand,
    },
    Ret,
    Syscall {
        number: u32,
    },
    Halt,
}

impl From<u32> for Instruction {
    fn from(value: u32) -> Self {
        todo!()
        // match value {
        //     0 => Self::gen(Instruction::Add {
        //         dest: todo!(),
        //         src1: todo!(),
        //         src2: todo!(),
        //     }),
        //     _ => Self::gen(Instruction::Add {
        //         dest: todo!(),
        //         src1: todo!(),
        //         src2: todo!(),
        //     }),
        // }
    }
}

pub trait InstructionHandler {
    fn fetch(&self, memory: u8) -> Instruction;

    fn decode(&self, opcode: Instruction) -> Result<(), ()>;
}
