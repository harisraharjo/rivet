pub mod register;

use register::Registers;

#[derive(Default, Debug)]
pub struct CPU {
    pub registers: Registers,
    pub pc: ProgramCounter,
    flags: u32,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Default::default(),
            pc: ProgramCounter::new(),
            flags: 0,
        }
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter(u32);

impl ProgramCounter {
    pub fn new() -> ProgramCounter {
        ProgramCounter(0)
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 4
    }

    #[inline(always)]
    pub fn value(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}
