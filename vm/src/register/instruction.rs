use macros::VMInstruction;

use super::Register;

// struct Opcode;
// impl Opcode {
//     fn new() -> Opcode {
//         Opcode{  }
//     }
// }

#[derive(Debug)]
pub enum Operand {
    Register(Register),
    Immediate(u32),
    Address(u32),
    LabelRef(u32), // For jump/call targets
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
    #[opcode(0x0)]
    Load { dest: Register, src: Operand },
    #[opcode(0xff)]
    Store { src: Register, dest: Operand },
    #[opcode(0xff)]
    Move { dest: Register, src: Operand },
    #[opcode(0xff)]
    Push { src: Operand },
    #[opcode(0xff)]
    Pop { dest: Register },
    #[opcode(0xff)]
    Add {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Sub {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Mul {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Div {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    And {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Or {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Xor {
        dest: Register,
        src1: Operand,
        src2: Operand,
    },
    #[opcode(0xff)]
    Shl {
        dest: Register,
        src: Operand,
        shift: Operand,
    },
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
    fn instruction_opcode() {
        let nop = Instruction::Nop;
        let halt = Instruction::Halt;
        let op1 = nop.opcode();
        let op2 = halt.opcode();
        println!("{op1}");
        println!("{op2}");
    }
}
