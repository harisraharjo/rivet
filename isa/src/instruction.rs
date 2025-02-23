use crate::{
    operand::{Immediate14, Immediate19},
    register::Register,
};
use shared::{DecodeError, EnumCount, EnumVariants, VMInstruction};

pub enum Rule {
    R3,
    R2I,
    RIR,
    RI,
}

pub enum InstructionType {
    /// Register - Register
    R, //r,r,r
    // Immediate type
    /// Immediate Arithmetic
    IA, //r,r,i
    /// Immediate Load
    IL, //r,i(r)
    /// Immediate Jump
    IJ, //r,r,i
    /// Store
    S, //r,i(r)
    /// Branch
    B, //r,r,i (asmbler: i is a label)
    //Jump
    J, //r,i
    /// Upper immediate
    U, //r,i
}

impl InstructionType {
    pub fn rule(&self) -> Rule {
        Rule::R2I
    }
}

impl From<u8> for InstructionType {
    fn from(value: u8) -> Self {
        // Safety: guaranteed to be safe because both have the same sizes (unit).
        unsafe { std::mem::transmute::<u8, InstructionType>(value) }
    }
}

impl From<InstructionType> for Rule {
    fn from(value: InstructionType) -> Self {
        match value {
            InstructionType::R => Self::R3,
            InstructionType::IA => Self::R2I,
            InstructionType::IL => Self::RIR,
            InstructionType::IJ => Self::R2I,
            InstructionType::S => Self::RIR,
            InstructionType::B => Self::R2I,
            InstructionType::J => Self::RI,
            InstructionType::U => Self::RI,
        }
    }
}

// impl From<&Instruction> for InstructionType {
//     fn from(value: &Instruction) -> Self {
//         match value {
//             Instruction::Add { ..} => todo!(),
//             Instruction::Sub { ..} => todo!(),
//             Instruction::Mul { ..} => todo!(),
//             Instruction::And { ..} => todo!(),
//             Instruction::Or { ..} => todo!(),
//             Instruction::Xor { ..} => todo!(),
//             Instruction::Shl { ..} => todo!(),
//             Instruction::Shr { ..} => todo!(),
//             Instruction::ShrA { ..} => todo!(),
//             Instruction::AddI { ..} => todo!(),
//             Instruction::Lui {..} => todo!(),
//             Instruction::Lw { ..} => todo!(),
//             Instruction::Sw { ..} => todo!(),
//             Instruction::Syscall { ..} => todo!(),
//             Instruction::Li {..} => todo!(),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, VMInstruction, EnumCount)]
// TODO: if fields got re-arranged, make sure to re-arrange the bit arrangements
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
    // Pseudo
    /// Load Immediate
    #[isa(0xff, 5, 19)]
    Li { dest: Register, value: Immediate19 },
    // #[isa(0xff,5,5,5)]
    // Syscall { number: u32 },
    // #[isa(0x0,5,5,5)]
    // Halt,
}

// impl Instruction {
// pub fn mnemonics(&self) -> i32 {
//     T::variants()
// }
// }

// pub struct ParseInstructionError(usize);
// impl From<usize> for Instruction {
//     fn from(value: usize) -> Self {
//         Self::VARIANT_COUNT
//     }
//     // type Error = ParseInstructionError;

//     // fn try_from(value: usize) -> Result<Self, Self::Error> {

//     // }
// }

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

#[cfg(test)]
mod test {
    use crate::{
        instruction::{self, Instruction},
        operand::{Immediate14, Immediate19},
        register::Register,
    };

    use shared::DecodeError;

    #[test]
    fn t_opcode() {
        // let d = instruction::Mnemonic
        let op1 = u32::from(&Instruction::Li {
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
            Instruction::Li {
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
