use std::marker::PhantomData;
use thiserror::Error;

use log::debug;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("out of bounds: `{0}`")]
    OutOfBounds(u32),
    #[error("out of bounds: `{0}`")]
    AddressTranslation(u32, Box<MemoryError>),
    #[error("no mapping: `{0}`")]
    NoMap(u32),
    #[error("invalid mapping index: `{0}`")]
    InvalidMap(u32, usize),
    #[error("out of bounds: `{0}`")]
    InternalMapperError(u32),
    #[error("internal mapper error @ `{0}`")]
    InternalMapperWithMessage(u32, String),
    #[error("this memory is read only")]
    ReadOnly,
}

pub trait Memory<T = u8> {
    fn read(&self, address: usize) -> Result<T, MemoryError>;
    fn write(&mut self, address: usize, value: T) -> Result<(), MemoryError>;
}

pub trait Load: Memory<u8> {
    fn load_program(&mut self, program: &[u8], start_address: usize);
    fn load_from_vec(&mut self, from: &[u8], addr: u32) -> Result<(), MemoryError> {
        for (i, b) in from.iter().enumerate() {
            self.write((addr as usize) + i, *b)?
        }
        Ok(())
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

    // pub
}

impl Memory<u8> for LinearMemory {
    fn read(&self, address: usize) -> Result<u8, MemoryError> {
        Ok(self.0[address])
    }

    fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        self.0[address] = value;
        Ok(())
    }
}

impl Memory<u16> for LinearMemory {
    fn read(&self, address: usize) -> Result<u16, MemoryError> {
        let bytes = &self.0[address..address + 2];
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn write(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        let bytes = value.to_le_bytes();
        self.0[address..address + 2].copy_from_slice(&bytes);
        Ok(())
    }
}

impl Memory<u32> for LinearMemory {
    fn read(&self, address: usize) -> Result<u32, MemoryError> {
        let bytes = &self.0[address..address + 4];
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn write(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        let bytes = value.to_le_bytes();
        self.0[address..address + 4].copy_from_slice(&bytes);
        Ok(())
    }
}

impl Load for LinearMemory {
    fn load_program(&mut self, program: &[u8], start_address: usize) {
        let end_address = start_address + program.len();
        self.0[start_address..end_address].copy_from_slice(program);
    }
}

pub struct Region<T, B = u8>
where
    T: Memory<B>,
{
    start: usize,
    size: usize,
    memory: T,
    __: PhantomData<B>,
}

impl<B, T: Memory<B>> Region<T, B> {
    pub fn new(start: usize, size: usize, memory: T) -> Region<T, B> {
        Region {
            start,
            size,
            memory,
            __: PhantomData,
        }
    }

    /// Check if the address is valid
    pub fn is_valid(&self, address: usize) -> bool {
        debug!(
            "start: {0}. address: {address}. size: {1}",
            self.start, self.size
        );

        self.start <= address && address < (self.size + self.start)
    }

    pub fn offset(&self, address: usize) -> usize {
        address - self.start
    }
}

pub struct LocalAddress {
    offset: usize,
    memory_index: usize,
}

#[derive(Default)]
pub struct MemoryManager<T, B = u8>
where
    T: Memory<B>,
{
    regions: Vec<Region<T, B>>,
}

impl<T> MemoryManager<T>
where
    T: Memory,
{
    pub fn new(start: usize, size: usize, memory: T) -> MemoryManager<T> {
        MemoryManager {
            regions: vec![Region::new(start, size, memory)],
        }
    }

    pub fn register(&mut self, start: usize, size: usize, memory: T) {
        self.regions.push(Region::new(start, size, memory));
    }

    /// Converts a virtual address into LocalAddress
    pub fn translate(&self, address: usize) -> Result<LocalAddress, MemoryError> {
        //could be sorted and use binary search
        match self
            .regions
            .iter()
            .enumerate()
            .find(move |(_, region)| region.is_valid(address))
        {
            Some((i, region)) => {
                let offset = region.offset(address);
                Ok(LocalAddress {
                    offset,
                    memory_index: i,
                })
            }
            None => Err(MemoryError::NoMap(address as u32)), // panic!("Invalid memory access: {:#08x}", address);
        }
    }
}

impl<T, B> Memory<B> for MemoryManager<T>
where
    T: Memory<B> + Memory,
{
    fn read(&self, address: usize) -> Result<B, MemoryError> {
        let local = self.translate(address)?;
        self.regions[local.memory_index].memory.read(local.offset)
    }

    fn write(&mut self, address: usize, value: B) -> Result<(), MemoryError> {
        let local = self.translate(address)?;
        self.regions[local.memory_index]
            .memory
            .write(local.offset, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_translation() {
        let memory_manager = MemoryManager::new(0x3000, 0x2000, LinearMemory::new(0x2000));
        // memory_manager.register(1, 2, LinearMemory::new(1024 * 5));

        // memory_manager.register(0x1000, 0x1000);  // 4KB chunk starting at 0x1000
        // memory_manager.register(0x3000, 0x2000);  // 8KB chunk starting at 0x3000
        // memory_manager.register(0x6000, 0x1000);

        // memory_manager.translate(0x1500); // Returns the first chunk with offset 0x500
        // memory_manager.translate(0x4000); // Returns the second chunk with offset 0x1000
        // memory_manager.translate(0x6100); // Returns the third chunk with offset 0x100
        // memory_manager.translate(0x2000); // Returns None (unmapped address)

        match memory_manager.translate(0x4000) {
            Ok(local) => {
                assert_eq!(local.offset, 0x1000)
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
