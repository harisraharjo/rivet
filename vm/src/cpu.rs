use std::ops::{Index, IndexMut};

use macros::EnumCount;

use crate::instruction::register::{ProgramCounter, Register};

#[derive(Default, Debug)]
pub struct Registers([u32; Register::VARIANT_COUNT]);

impl Registers {
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

impl Index<Register> for Registers {
    type Output = u32;

    fn index(&self, index: Register) -> &Self::Output {
        todo!()
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        todo!()
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
