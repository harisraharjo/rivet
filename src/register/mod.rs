pub mod instruction;

#[derive(Debug)]
pub enum Register {
    Zero,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
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
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::BP,
            8 => Register::SP,
            9 => Register::RA,
            _ => Register::Zero,
        }
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    pub fn new() -> ProgramCounter {
        ProgramCounter(0)
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 4
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        self.0
    }
}
