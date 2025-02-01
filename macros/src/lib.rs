pub use macros_derive::{EnumCount, VMInstruction};

pub trait VMInstruction {
    fn opcode(&self) -> u8;
    // fn instruction_format(&self, bytes: u32) -> i32;
}

pub trait EnumCount {
    const VARIANT_COUNT: usize;
}
