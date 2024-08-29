pub use macros_derive::{EnumCount, VMInstruction};

pub trait VMInstruction {
    fn opcode(&self) -> u8;
}

pub trait EnumCount {
    const VARIANT_COUNT: usize;
}
