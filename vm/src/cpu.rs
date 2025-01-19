use macros::EnumCount;

use crate::instruction::register::{ProgramCounter, Register};

#[derive(Default, Debug)]
pub struct Registers([u32; Register::VARIANT_COUNT]);

impl Registers {
    // fn new() -> Registers {
    //     Registers{  }
    // }

    pub fn get(&self, register: Register) -> u32 {
        self.0[register as usize]
    }

    pub fn set(&mut self, register: Register, value: u32) {
        self.0[register as usize] = value;
        // Ok(())
    }

    pub fn reset(&mut self) {
        self.0 = [0; Register::VARIANT_COUNT];
    }
}

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
