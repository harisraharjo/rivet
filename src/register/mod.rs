pub mod instruction;
pub mod pc;

pub enum Register {
    Zero,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    /// Stack Pointer
    SP,
    /// Base Pointer
    BP,
    /// Return Address
    RA,
}

impl Register {
    // TODO: Create me a macro
    pub const fn count() -> usize {
        10
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value {
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::BP,
            8 => Register::SP,
            9 => Register::RA,
            _ => Register::Zero,
        }
    }
}
