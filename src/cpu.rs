use crate::register::{ProgramCounter, Register};

#[derive(Default, Debug)]
pub struct Registers([u32; Register::count()]);

impl Registers {
    // fn new() -> Registers {
    //     Registers{  }
    // }

    pub fn get(&self, register: Register) -> u32 {
        self.0[register as usize]
    }

    pub fn set(&mut self, register: Register, value: u32) -> Result<(), ()> {
        self.0[register as usize] = value;
        Ok(())
    }
}

pub trait PC {}

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
