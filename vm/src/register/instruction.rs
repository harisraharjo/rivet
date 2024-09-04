use std::ops::{BitAnd, Shr};

use macros::VMInstruction;

use super::Register;

#[derive(Debug)]
pub enum Operand {
    Register(Register),
    Immediate(u32),
    Address(u32),
    LabelRef(u32), // For jump/call targets
}

impl BitAnd<u32> for Operand {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        match self {
            Operand::Register(data) => data & rhs,
            Operand::Immediate(data) => data & rhs,
            Operand::Address(data) => data & rhs,
            Operand::LabelRef(data) => data & rhs,
        }
    }
}

impl From<u32> for Operand {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Register(value.into()),
            1 => Self::Immediate(value),
            2 => Self::Address(value),
            _ => Self::LabelRef(0),
        }
    }
}

impl From<u16> for Operand {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Register(value.into()),
            1 => Self::Immediate(value.into()),
            2 => Self::Address(value.into()),
            _ => Self::LabelRef(0),
        }
    }
}

impl From<u8> for Operand {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Register(value.into()),
            1 => Self::Immediate(value.into()),
            2 => Self::Address(value.into()),
            _ => Self::LabelRef(0),
        }
    }
}

#[derive(Debug, VMInstruction)]
pub enum Instruction {
    #[opcode(0xff)]
    Nop,
    // ---Binary Operators---
    #[opcode(0x1)]
    Add {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0x2)]
    Sub {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0x3)]
    Mul {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0x4)]
    And {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0x5)]
    Or {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0x6)]
    Xor {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    // #[opcode(0xff)]
    // Div {
    //     dest: Register,
    //     src1: Operand,
    //     src2: Operand,
    // },

    // ---Load and Store---
    #[opcode(0xc)]
    LoadWord { dest: Register, src: Operand },
    #[opcode(0xd)]
    StoreWord { src: Register, dest: Operand },
    #[opcode(0xff)]
    Move { dest: Register, src: Operand },
    #[opcode(0xff)]
    Push { src: Operand },
    #[opcode(0xff)]
    Pop { dest: Register },
    /// Shift Left
    #[opcode(0xff)]
    Shl {
        dest: Register,
        src: Operand,
        shift: Operand,
    },
    /// Shift Right
    #[opcode(0xff)]
    Shr {
        dest: Register,
        src: Operand,
        shift: Operand,
    },
    #[opcode(0xff)]
    Cmp { left: Operand, right: Operand },
    #[opcode(0xff)]
    Jmp { target: Operand },
    // #[opcode(0xff)]
    // Je { target: Operand },
    // #[opcode(0xff)]
    // Jne { target: Operand },
    // #[opcode(0xff)]
    // Jg { target: Operand },
    // #[opcode(0xff)]
    // Jl { target: Operand },
    #[opcode(0xff)]
    Call { target: Operand },
    #[opcode(0xff)]
    Ret,
    #[opcode(0xff)]
    Syscall { number: u32 },
    #[opcode(0x0)]
    Halt,
}

// pub trait InstructionHandler {
//     fn fetch(&self, memory: u8) -> Instruction;

//     fn decode(&self, opcode: Instruction) -> Result<(), ()>;
// }

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_opcode() {
        let nop = Instruction::Nop;
        let halt = Instruction::Halt;
        // let op1 = nop.opcode();
        // let op2 = halt.opcode();
        // println!("{op1}");
        // println!("{op2}");
    }

    #[test]
    fn test_encodings() {
        let ops: Vec<Instruction> = vec![
            Instruction::Add {
                dest: todo!(),
                src1: todo!(),
                src2: todo!(),
            },
            Instruction::Sub {
                dest: todo!(),
                src1: todo!(),
                src2: todo!(),
            },
            Instruction::Mul {
                dest: todo!(),
                src1: todo!(),
                src2: todo!(),
            },
            Instruction::And {
                dest: todo!(),
                src1: todo!(),
                src2: todo!(),
            },
            Instruction::Or {
                dest: todo!(),
                src1: todo!(),
                src2: todo!(),
            },
        ];

        // let encoded: Vec<_> = ops.iter().map(|x| x.encode_u16()).collect();
        // for (l, r) in ops.iter().zip(encoded.iter()) {
        //     assert_eq!(*l, Instruction::try_from(*r)?);
        // }
        // Ok(())
    }
}
