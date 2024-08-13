pub mod instruction;
pub mod pc;

pub enum Register {
    ZERO,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    PC,
    // Stack Pointer
    SP,
    BP,
}

#[derive(Default, Debug)]
pub struct Registers {
    gprs: [u32; 8],
    pc: u32,
    flags: u32,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            gprs: Default::default(),
            pc: 0,
            flags: 0,
        }
    }

    pub fn get(&self, register: Register) -> u32 {
        self.gprs[register as usize]
    }

    pub fn set(&mut self, register: Register, value: u32) -> Result<(), ()> {
        self.gprs[register as usize] = value;
        Ok(())
    }
}
