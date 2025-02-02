pub use macros_derive::{EnumCount, VMInstruction};

pub trait VMInstruction {
    fn opcode(&self) -> u8;
    // fn instruction_format(&self, bytes: u32) -> i32;
}

pub trait EnumCount {
    const VARIANT_COUNT: usize;
}

pub mod helper {
    pub fn max_value_for_bit_length<const SIGNED: bool>(bit_count: u32) -> u32 {
        if !SIGNED {
            (1u32 << bit_count) - 1
        } else {
            (1 << (bit_count - 1)) - 1
        }
    }
}
