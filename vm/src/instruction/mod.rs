pub mod operand;
pub mod register;

use macros::VMInstruction;
use register::Register;
// #[derive(Debug, PartialEq, Eq)]
// pub enum Operand {
//     Register(Register),
//     Immediate(i32),
//     Address(u32),
//     LabelRef(u32), // For jump/call targets
// }

#[derive(Debug, PartialEq, Eq, VMInstruction)]
pub enum Instruction {
    #[opcode(0xff)]
    Nop,
    // ---Binary Operators---
    #[opcode(0x1)]
    Add {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x2)]
    Sub {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x3)]
    Mul {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x4)]
    And {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x5)]
    Or {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x6)]
    Xor {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[opcode(0x13)]
    AddI {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    // #[opcode(0xff)]
    // Div {
    //     dest: Register,
    //     src1: Register,
    //     src2: Register,
    // },

    // ---Load and Store---
    #[opcode(0xc)]
    LoadWord { dest: Register, src: Register },
    #[opcode(0xd)]
    StoreWord { src: Register, dest: Register },
    #[opcode(0xff)]
    Move { dest: Register, src: Register },
    // #[opcode(0xff)]
    // Push { src: Register },
    // #[opcode(0xff)]
    // Pop { dest: Register },
    /// Shift Left
    #[opcode(0xff)]
    Shl {
        dest: Register,
        src: Register,
        shift: Register,
    },
    /// Shift Right
    #[opcode(0xff)]
    Shr {
        dest: Register,
        src: Register,
        shift: Register,
    },
    #[opcode(0xff)]
    Jal {
        dest: Register,
        src: Register,
        shift: Register,
    },
    // #[opcode(0xff)]
    // Cmp { left: Register, right: Register },
    // #[opcode(0xff)]
    // Jmp { target: Register },
    // #[opcode(0xff)]
    // Je { target: Register },
    // #[opcode(0xff)]
    // Jne { target: Register },
    // #[opcode(0xff)]
    // Jg { target: Register },
    // #[opcode(0xff)]
    // Jl { target: Register },
    // #[opcode(0xff)]
    // Call { target: Register },
    // #[opcode(0xff)]
    // Ret,
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
    use std::mem::transmute;

    use super::*;
    use register::*;
    #[test]
    fn test_opcode() {
        let op1 = u32::from(&Instruction::Nop) as u8;
        let op2 = u32::from(&Instruction::Halt) as u8;
        // let op3 = u32::from(&Instruction::Add {
        //     dest: Register::BP,
        //     src1: Register::Register(Register::X1),
        //     src2: Operand::Register(Register::X4),
        // }) as u8;

        assert_eq!(op1.to_le_bytes(), 0xff_u8.to_le_bytes());
        assert_eq!(op2.to_le_bytes(), 0x0_u8.to_le_bytes());
        // assert_eq!(op3.to_le_bytes(), 0x1_u8.to_le_bytes());
    }

    #[test]
    fn test_encodings() -> Result<(), DecodeError> {
        println!("TEST ENCODE START");
        let ops: Vec<Instruction> = vec![
            Instruction::Add {
                dest: Register::X2,
                src1: Register::X3,
                src2: Register::X4,
            },
            // Instruction::Sub {
            //     dest: Register::BP,
            //     src1: Operand::Register(Register::X2),
            //     src2: Operand::Register(Register::X5),
            // },
            // Instruction::Mul {
            //     dest: Register::BP,
            //     src1: Operand::Register(Register::X3),
            //     src2: Operand::Register(Register::X1),
            // },
            // Instruction::And {
            //     dest: Register::BP,
            //     src1: Operand::Register(Register::X6),
            //     src2: Operand::Register(Register::X3),
            // },
            // Instruction::Or {
            //     dest: Register::BP,
            //     src1: Operand::Register(Register::X4),
            //     src2: Operand::Register(Register::X6),
            // },
        ];
        // if (ins & 0x8000) == 0 {

        println!("{}", 0xff);
        println!("{}", 0xfff);
        let dest_val: u8 = unsafe { transmute(Register::X2) };
        let src1_val: u8 = unsafe { transmute(Register::X3) };
        let src2_val: u8 = unsafe { transmute(Register::X4) };

        let encoded: u32 = (1u8 as u32)
            | ((dest_val as u32) << 8)
            | ((src1_val as u32) << 16)
            | ((src2_val as u32) << 24);
        println!("MANUAL ENCODING u32: {:?}", encoded);

        let dest = unsafe { transmute::<u8, Register>((encoded >> 8) as u8) };
        let src1 = unsafe { transmute::<u8, Register>((encoded >> 16) as u8) };
        let src2 = unsafe { transmute::<u8, Register>((encoded >> 24) as u8) };

        let result = Instruction::Add { dest, src1, src2 };
        // let result = Instruction::try_from(result).unwrap();

        println!("MANUAL ENCODING Instruction: {:?}", result);

        let encoded: Vec<u32> = ops
            .iter()
            .map(|x| {
                let ff = x;
                ff.into()
            })
            .collect();
        for (l, r) in ops.iter().zip(encoded.iter()) {
            let decoded = Instruction::try_from(*r)?;
            // println!("{:?}", l);
            println!("{:?}", r);
            // println!("{:?}", decoded);
            assert_eq!(*l, decoded);
        }
        Ok(())
    }
}
