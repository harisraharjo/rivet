use crate::register::Register;

#[derive(Default, Debug)]
pub struct CPU {
    registers: [u32; Register::count()],
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

    pub fn get(&self, register: Register) -> u32 {
        self.registers[register as usize]
    }

    pub fn set(&mut self, register: Register, value: u32) -> Result<(), ()> {
        self.registers[register as usize] = value;
        Ok(())
    }
}
