pub use macros_derive::{EnumCount, EnumVariants, VMInstruction};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Unknown Opcode: `{0}`")]
    UnknownOpcode(u8),
}

pub trait VMInstruction {
    fn opcode(&self) -> u8;
    // fn instruction_format(&self, bytes: u32) -> i32;
}

pub trait EnumCount {
    const VARIANT_COUNT: usize;
}

pub trait EnumVariants<const N: usize> {
    fn variants<'a>() -> [&'a str; N];
}

pub const fn max_value_for_bit_length<const SIGNED: bool>(bit_count: u32) -> u32 {
    if !SIGNED {
        (1u32 << bit_count) - 1
    } else {
        (1 << (bit_count - 1)) - 1
    }
}
