// use std::{marker::PhantomData, ops::Range};

use thiserror::Error;

// use log::debug;

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

pub trait ReadWrite<T> {
    fn read(&self, address: usize) -> Result<T, MemoryError>;
    fn write(&mut self, address: usize, value: T) -> Result<(), MemoryError>;
}

pub trait Load: ReadWrite<u8> {
    fn load_program(&mut self, program: &[u8], start_address: usize);
    fn load_from_vec(&mut self, program: &[u8], addr: u32) -> Result<(), MemoryError> {
        for (i, b) in program.iter().enumerate() {
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
}

impl ReadWrite<u8> for LinearMemory {
    fn read(&self, address: usize) -> Result<u8, MemoryError> {
        Ok(self.0[address])
    }

    fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        self.0[address] = value;
        Ok(())
    }
}

impl ReadWrite<u16> for LinearMemory {
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

impl ReadWrite<u32> for LinearMemory {
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

enum Permission {
    R,
    W,
    X,
}

enum Region {
    Code,
    Data,
    Stack,
    Heap,
}

impl Region {
    // /// Check if the address is valid
    // pub fn is_valid(&self, address: usize) -> bool {
    //     debug!(
    //         "start: {0}. address: {address}. size: {1}",
    //         self.start, self.size
    //     );

    //     // self.start <= address && address < (self.size + self.start)
    // }

    pub const fn range(&self) -> (u32, u32) {
        match self {
            Region::Code => (0x0000_0000, 0x0000_1000),
            Region::Data => (0x0000_1000, 0x4000_0000),
            Region::Heap => (0x4000_0000, 0x8000_0000),
            Region::Stack => (0x8000_0000, 0xFFFF_FFFF),
        }
    }

    pub const fn permissions(&self) -> &[Permission] {
        match self {
            Region::Data => &[Permission::R, Permission::X],
            Region::Code => &[Permission::R, Permission::X],
            Region::Heap => &[Permission::R, Permission::X, Permission::W],
            Region::Stack => &[Permission::R, Permission::X, Permission::W],
        }
    }

    // pub fn offset(&self, address: usize) -> usize {
    //     address - self.start
    // }
}

pub struct Regions {
    // members: [Region; 4], // Example: code [0x0, 0x1000), heap [0x1000, 0x8000), stack [0x8000, 0xFFFF_FFFF]
    code: Region,
    data: Region,
    heap: Region,
    stack: Region,
}

impl Regions {
    fn new() -> Regions {
        Regions {
            code: Region::Code,
            data: Region::Data,
            heap: Region::Heap,
            stack: Region::Stack,
        }
    }
}

#[derive(Debug)]
pub struct LocalAddress {
    offset: usize,
    memory_index: usize,
}

pub struct MemoryManager<M> {
    memory: M,
    regions: Regions,
}

impl<M> MemoryManager<M> {
    pub fn new(memory: M) -> MemoryManager<M> {
        MemoryManager {
            memory,
            regions: Regions::new(),
        }
    }

    /// Validate address and alignment, return buffer offset
    fn validate(
        &self,
        vaddr: u32,
        size: u32, // 1, 2, or 4 bytes
        is_write: bool,
    ) -> Result<usize, &'static str> {
        println!("VALIDATING..");
        // Alignment check (RISC-V requires alignment for LW/SW/LH/SH)
        if vaddr % size != 0 {
            return Err("Unaligned access");
        }

        let r = self.regions.code.range();
        println!("Code segment start: {}, end: {}", r.0, r.1);
        // Code segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            if is_write {
                return Err("Write to code segment");
            }
            return Ok(vaddr as usize);
        }

        let r = self.regions.code.range();
        println!("Data segment start: {}, end: {}", r.0, r.1);
        // Data segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            if is_write {
                return Err("Write to data segment");
            }
            return Ok(vaddr as usize);
        }

        let r = self.regions.heap.range();
        println!("Heap segment start: {}, end: {}", r.0, r.1);
        // Heap segment (read/write)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            return Ok(vaddr as usize);
        }

        let r = self.regions.stack.range();
        println!("Stack segment start: {}, end: {}", r.0, r.1);
        // Stack segment (read/write, grows downward)
        if vaddr <= r.0 && vaddr >= r.1 {
            return Ok(vaddr as usize);
        }

        Err("Address out of bounds")
    }

    pub fn read<T>(&self, address: u32) -> Result<T, MemoryError>
    where
        M: ReadWrite<T>,
    {
        // TODO: TIDY ME
        let real_addr = self.validate(address, 4, false).unwrap();
        println!("Memory Data: {real_addr}");
        self.memory.read(real_addr)
    }

    pub fn write<T>(&mut self, address: u32, value: T) -> Result<(), MemoryError>
    where
        T: Copy,
        M: ReadWrite<T>,
    {
        let real_addr = self.validate(address, 4, true).unwrap();
        self.memory.write(real_addr, value)
    }

    // pub fn register(&mut self, start: usize, size: usize, memory: M) {
    //     self.regions.push(Region::new(start, size, memory));
    // }

    // /// Converts a virtual address into LocalAddress
    // pub fn translate(&self, address: usize) -> Result<LocalAddress, MemoryError> {
    //     //could be sorted and use binary search
    //     match self
    //         .regions
    //         .iter()
    //         .enumerate()
    //         .find(move |(_, region)| region.is_valid(address))
    //     {
    //         Some((i, region)) => {
    //             let offset = region.offset(address);
    //             Ok(LocalAddress {
    //                 offset,
    //                 memory_index: i,
    //             })
    //         }
    //         None => Err(MemoryError::NoMap(address as u32)), // panic!("Invalid memory access: {:#08x}", address);
    //     }
    // }

    #[cfg(test)]
    pub fn load_from_vec_delete_me_later<T>(
        &mut self,
        program: &[T],
        addr: u32,
    ) -> Result<(), MemoryError>
    where
        T: Copy,
        M: ReadWrite<T>,
    {
        for (i, b) in program.iter().enumerate() {
            self.memory.write((addr as usize) + i, *b)?
        }
        Ok(())
    }

    // Inherent `read` method
    // fn read<T>(&self, address: usize) -> Result<T, MemoryError>
    // where
    //     Self: Addressable + ReadWrite<T>, // Ensure MyStruct implements ReadWrite<T>
    // {
    //     // Fully qualified syntax to call the trait's `read` method
    //     <Self as ReadWrite<u8>>::read(self, address)

    // let local = self.translate(address)?;
    // // self.regions[local.memory_index].memory.read(local.offset)
    // let ff = ReadWrite::<T>::read(&self.regions[local.memory_index].memory, local.offset);
    // }
}

// impl<M> ReadWrite<u8> for MemoryManager<M>
// where
//     M: Addressable + ReadWrite,
// {
//     fn read(&self, address: usize) -> Result<u8, MemoryError> {
//         // let local = self.translate(address)?;

//         // //         const IO_BASE_ADDRESS: usize = 0x1000; // Base address for memory-mapped I/O
//         // // const IO_SIZE: usize = 0x1000; // Size of the I/O region (4 KB)

//         //         // Regular memory
//         //             let physical_address = self.translate(address);
//         //             let bytes = [
//         //                 self.physical_memory[physical_address],
//         //                 self.physical_memory[physical_address + 1],
//         //                 self.physical_memory[physical_address + 2],
//         //                 self.physical_memory[physical_address + 3],
//         //             ];
//         //             u32::from_le_bytes(bytes)

//         // self.regions[local.memory_index].memory.read(local.offset)
//         self.memory.read(address)
//     }

//     fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
//         // let local = self.translate(address)?;
//         // self.regions[local.memory_index]
//         //     .memory
//         //     .write(local.offset, value)
//         self.memory.write(address, value)
//     }
// }

// impl<M> ReadWrite<u32> for MemoryManager<M>
// where
//     M: Addressable + ReadWrite + ReadWrite<u32>,
// {
//     fn read(&self, address: usize) -> Result<u32, MemoryError> {
//         // let local = self.translate(address)?;
//         // println!("virtual: {address}, local: {:#?}", local);
//         // self.regions[local.memory_index].memory.read(local.offset)

//         self.memory.read(address)
//         // ReadWrite::<u32>::read(&self.regions[local.memory_index].memory, local.offset)
//     }

//     fn write(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t_memory_translation() {
        // let memory_manager = MemoryManager::new(0x3000, 0x2000, LinearMemory::new(0x2000));
        // // memory_manager.register(1, 2, LinearMemory::new(1024 * 5));

        // // memory_manager.register(0x1000, 0x1000);  // 4KB chunk starting at 0x1000
        // // memory_manager.register(0x3000, 0x2000);  // 8KB chunk starting at 0x3000
        // // memory_manager.register(0x6000, 0x1000);

        // // memory_manager.translate(0x1500); // Returns the first chunk with offset 0x500
        // // memory_manager.translate(0x4000); // Returns the second chunk with offset 0x1000
        // // memory_manager.translate(0x6100); // Returns the third chunk with offset 0x100
        // // memory_manager.translate(0x2000); // Returns None (unmapped address)

        // match memory_manager.translate(0x4000) {
        //     Ok(local) => {
        //         assert_eq!(local.offset, 0x1000)
        //     }
        //     Err(e) => eprintln!("{e}"),
        // }
    }
}
