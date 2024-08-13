#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds(u32),
    AddressTranslation(u32, Box<MemoryError>),
    NoMap(u32),
    InvalidMap(u32, usize),
    InternalMapperError(u32),
    InternalMapperWithMessage(u32, String),
}

pub trait Memory {
    fn read(&self, addr: u32) -> Result<u8, MemoryError>;
    fn write(&mut self, addr: u32, value: u8) -> Result<(), MemoryError>;

    fn read2(&mut self, addr: u32) -> Result<u16, MemoryError> {
        let x0 = self.read(addr)?;
        let x1 = self.read(addr + 1)?;
        Ok((x0 as u16) | ((x1 as u16) << 8))
    }

    fn write2(&mut self, addr: u32, value: u16) -> Result<(), MemoryError> {
        let lower = value & 0xff;
        let upper = (value & 0xff00) >> 8;
        self.write(addr, lower as u8)?;
        self.write(addr + 1, upper as u8)
    }

    fn copy(&mut self, from: u32, to: u32, n: usize) -> Result<bool, ()> {
        // for i in 0..n {
        //     if let Some(bit) = self.read(from + (i as u16)) {
        //         if self.write(to + (i as u16), bit).is_err() {
        //             return Ok(false);
        //         }
        //     } else {
        //         return Ok(false);
        //     }
        // }
        todo!()
    }
}

pub struct LinearMemory(Vec<u8>);
impl LinearMemory {
    pub fn new(size: usize) -> LinearMemory {
        LinearMemory(vec![0; size])
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}
