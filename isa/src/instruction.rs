use crate::{
    operand::{Immediate14, Immediate19},
    register::Register,
};
use shared::{DecodeError, EnumCount, EnumVariants, VMInstruction};

#[derive(Debug, PartialEq, Eq, VMInstruction, EnumCount)]
// TODO: if fields got re-arranged, make sure to also re-arrange the bits e.g `(..5, 5, 5)`
pub enum Instruction {
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
    /// Add Immediate
    #[isa(0x13, 5, 5, 14)]
    AddI {
        dest: Register,
        src: Register,
        value: Immediate14,
    },
    /// Load Upper Immediate.
    #[isa(0x14, 5, 19)]
    Lui { dest: Register, value: Immediate19 },
    /// Load Word
    #[isa(0xc, 5, 5, 14)]
    Lw {
        dest: Register,
        src: Register,
        offset: Immediate14,
    },
    /// Store Word
    #[isa(0xd, 5, 5, 14)]
    Sw {
        src: Register,
        dest: Register,
        offset: Immediate14,
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

pub trait Codec {
    fn decode(src: u32, bit_accumulation: u32, bit_mask: u32) -> Self
    where
        Self: core::convert::From<u32>,
    {
        ((src >> bit_accumulation) & bit_mask).into()
    }

    fn encode(&self, bit_mask: u32, bit_accumulation: u32) -> u32
    where
        for<'a> &'a Self: std::ops::BitAnd<u32, Output = u32> + std::ops::Shl<u32, Output = u32>,
    {
        (self & bit_mask) << bit_accumulation
    }
}

#[derive(Debug)]
pub enum InstructionType {
    /// Register - Register
    Arithmetic, //r,r,r
    /// Immediate Arithmetic
    IA, //r,r,i
    /// Branch
    B, //r,r,i (asmbler: i is a label)
    /// Immediate Jump
    IJ, //r,r,i
    /// Immediate Load
    IL, //r,i(r)
    /// Store
    S, //r,i(r)
    //Jump
    J, //r,i
    /// Upper immediate
    U, //r,i
}

#[cfg(test)]
mod test {
    use crate::{
        instruction::Instruction,
        operand::{Immediate14, Immediate19},
        register::Register,
    };

    use shared::DecodeError;

    #[test]
    fn t_opcode() {
        // let d = instruction::Mnemonic
        let op1 = u32::from(&Instruction::Lui {
            dest: Register::X0,
            value: Immediate19::new(150),
        }) as u8;

        assert_eq!(op1.to_le_bytes(), 0xff_u8.to_le_bytes());
    }

    #[test]
    fn t_encode_decode() -> Result<(), DecodeError> {
        let ins: Vec<Instruction> = vec![
            Instruction::Add {
                dest: Register::X10,
                src1: Register::X11,
                src2: Register::X12,
            },
            Instruction::Lui {
                dest: Register::X5,
                value: Immediate19::new(150),
            },
            Instruction::Lui {
                dest: Register::X5,
                value: Immediate19::new(150),
            },
            Instruction::AddI {
                dest: Register::X10,
                src: Register::X11,
                value: Immediate14::new(-31),
            },
            // Instruction::LoadWord {
            //     dest: Register::X7,
            //     src: Register::X28,
            //     offset: Immediate::new(11),
            // },
            // Instruction::StoreWord {
            //     dest: Register::X7,
            //     src: Register::X28,
            //     offset: Immediate::new(11),
            // },
            Instruction::Syscall {
                src1: Register::X11,
                src2: Register::X12,
                src3: Register::X13,
            },
        ];

        let encoded: Vec<u32> = ins.iter().map(|x| x.into()).collect();
        for (l, r) in ins.iter().zip(encoded.iter()) {
            println!("instruction: {:?}, memory representation: {1}", l, r);
            let decoded = Instruction::try_from(*r)?;
            assert_eq!(*l, decoded);
        }
        Ok(())
    }
}
