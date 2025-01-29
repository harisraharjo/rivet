pub mod operand;
pub mod register;

use macros::VMInstruction;
use register::Register;

#[derive(Debug, PartialEq, Eq, VMInstruction)]
pub enum Instruction {
    #[isa(0xff, 5, 19)]
    Li { dest: Register, value: u32 },
    // ---Binary Operators---
    #[isa(0x1, 5, 5, 5)]
    Add {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[isa(0x2, 5, 5, 5)]
    Sub {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[isa(0x3, 5, 5, 5)]
    Mul {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[isa(0x4, 5, 5, 5)]
    And {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[isa(0x5, 5, 5, 5)]
    Or {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    #[isa(0x6, 5, 5, 5)]
    Xor {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    /// Shift Left
    #[isa(0x7, 5, 5, 5)]
    Shl {
        dest: Register,
        src: Register,
        shift: Register,
    },
    /// Shift Right Logical
    #[isa(0x8, 5, 5, 5)]
    Shr {
        dest: Register,
        src: Register,
        shift: Register,
    },
    /// Shift Right Arith
    #[isa(0x9, 5, 5, 5)]
    ShrA {
        dest: Register,
        src: Register,
        shift: Register,
    },
    // --- Imm ---
    #[isa(0x13, 5, 5, 14)]
    AddI {
        dest: Register,
        src: Register,
        value: u16,
    },
    #[isa(0xc, 5, 5, 14)]
    LoadWord {
        dest: Register,
        src: Register,
        offset: u16,
    },
    #[isa(0xd, 5, 5, 14)]
    StoreWord {
        dest: Register,
        src: Register,
        offset: u16,
    },
    // #[isa(0xe,5,5,5)]
    // LoadByte {
    //     dest: Register,
    //     src: Register,
    //     offset: u8,
    // },
    // #[isa(0xff,5,5,5)]
    // Jal {
    //     dest: Register,
    //     // offset:
    // },
    // TODO: Exit, halt, shutdown
    // pub const SIGHALT: u8 = 0xf;
    // #[isa(0X5D,5,5,5)]
    #[isa(0x73, 5, 5, 5)]
    Syscall {
        src1: Register,
        src2: Register,
        src3: Register,
    },
    // #[isa(0xff,5,5,5)]
    // Syscall { number: u32 },
    // #[isa(0x0,5,5,5)]
    // Halt,
}

// pub trait InstructionHandler {
//     fn fetch(&self, memory: u8) -> Instruction;

//     fn decode(&self, opcode: Instruction) -> Result<(), ()>;
// }

#[cfg(test)]
mod test {
    use super::*;
    use register::*;

    #[test]
    fn t_opcode() {
        let op1 = u32::from(&Instruction::Li {
            dest: Register::Zero,
            value: 150,
        }) as u8;

        assert_eq!(op1.to_le_bytes(), 0xff_u8.to_le_bytes());
    }

    #[test]
    fn t_overflow_todo() {
        // TODO: TEST OVERFLOW IN THE ASSEMBLER NOT HERE
    }

    #[test]
    fn t_encode_decode() -> Result<(), DecodeError> {
        let ops: Vec<Instruction> = vec![
            Instruction::Add {
                dest: Register::A0,
                src1: Register::A1,
                src2: Register::A2,
            },
            // Instruction::LoadWord {
            //     dest: Register::X4,
            //     src: Register::X10,
            //     offset: 213,
            // },
            // Instruction::StoreWord {
            //     dest: Register::X4,
            //     src: Register::X9,
            //     offset: 255,
            // },
            Instruction::Li {
                dest: Register::T0,
                value: 150,
            },
            Instruction::AddI {
                dest: Register::A0,
                src: Register::A1,
                value: 13,
            },
            Instruction::Syscall {
                src1: Register::A1,
                src2: Register::A2,
                src3: Register::A3,
            },
        ];
        // if (ins & 0x8000) == 0 {

        // println!("{}", 0xff);
        // println!("{}", 0xfff);
        // let dest_val: u8 = unsafe { transmute(Register::X2) };
        // let src1_val: u8 = unsafe { transmute(Register::X3) };
        // let src2_val: u8 = unsafe { transmute(Register::X4) };

        // let encoded: u32 = (1u8 as u32)
        //     | ((dest_val as u32) << 8)
        //     | ((src1_val as u32) << 16)
        //     | ((src2_val as u32) << 24);
        // println!("MANUAL ENCODING u32: {:?}", encoded);

        // let dest = unsafe { transmute::<u8, Register>((encoded >> 8) as u8) };
        // let src1 = unsafe { transmute::<u8, Register>((encoded >> 16) as u8) };
        // let src2 = unsafe { transmute::<u8, Register>((encoded >> 24) as u8) };

        // let result = Instruction::Add { dest, src1, src2 };
        // // let result = Instruction::try_from(result).unwrap();

        // println!("MANUAL ENCODING Instruction: {:?}", result);

        // let a1 = 10 & 11;
        // let a2 = 11 & 10;
        // println!("A1: {a1}");
        // println!("A2: {a2}");

        let encoded: Vec<u32> = ops.iter().map(|x| x.into()).collect();
        for (i, (l, r)) in ops.iter().zip(encoded.iter()).enumerate() {
            println!("{i} instruction: {:?}, memory representation: {1}", l, r);
            let decoded = Instruction::try_from(*r)?;
            println!("Decoded: {:?}", decoded);
            assert_eq!(*l, decoded);
        }
        Ok(())
    }
}
