#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds(u32),
    AddressTranslation(u32, Box<MemoryError>),
    NoMap(u32),
    InvalidMap(u32, usize),
    InternalMapperError(u32),
    InternalMapperWithMessage(u32, String),
}

pub trait Mem<T = u8> {
    fn read(&self, address: usize) -> Result<T, MemoryError>;
    fn write(&mut self, address: usize, value: T) -> Result<(), MemoryError>;
}

// pub trait Memory {
//     fn read(&self, address: usize) -> Result<u8, MemoryError>;
//     fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError>;

//     fn read2(&mut self, address: usize) -> Result<u16, MemoryError>;

//     fn write2(&mut self, address: usize, value: u16) -> Result<(), MemoryError>;

//     fn read3(&mut self, address: usize) -> Result<u32, MemoryError>;

//     fn write3(&mut self, address: usize, value: u32) -> Result<(), MemoryError>;

//     fn copy(&mut self, from: u32, to: u32, n: usize) -> Result<bool, ()>;
// }

pub struct LinearMemory(Vec<u8>);
impl LinearMemory {
    pub fn new(size: usize) -> LinearMemory {
        LinearMemory(vec![0; size])
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl Mem<u8> for LinearMemory {
    fn read(&self, address: usize) -> Result<u8, MemoryError> {
        Ok(self.0[address])
    }

    fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        todo!()
    }
}

impl Mem<u16> for LinearMemory {
    fn read(&self, address: usize) -> Result<u16, MemoryError> {
        let bytes = &self.0[address..address + 2];
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn write(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        todo!()
    }
}

impl Mem<u32> for LinearMemory {
    fn read(&self, address: usize) -> Result<u32, MemoryError> {
        let bytes = &self.0[address..address + 4];
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn write(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        todo!()
    }
}
