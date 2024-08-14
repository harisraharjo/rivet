use crate::register::Register;

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

#[derive(Default, Debug)]
pub struct CPU {
    pub registers: Registers,
    pc: u32,
    flags: u32,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Default::default(),
            pc: 0,
            flags: 0,
        }
    }
}
