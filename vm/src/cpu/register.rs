use std::ops::{Index, IndexMut};

use isa::Register;
use shared::EnumCount;

#[derive(Default, Debug)]
pub struct Registers([u32; Register::VARIANT_COUNT]);

impl Registers {
    pub fn get(&self, register: Register) -> u32 {
        self.0[register as usize]
    }

    pub fn set(&mut self, register: Register, value: u32) {
        self.0[register as usize] = value;
    }

    pub fn reset(&mut self) {
        self.0.fill(0);
    }
}

impl Index<Register> for Registers {
    type Output = u32;

    fn index(&self, index: Register) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
