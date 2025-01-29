// use std::{marker::PhantomData, ops::Range};

use std::ops::{Index, IndexMut};

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
    // fn bulk_write(&self, address: usize, value: T) -> i32 {
    //     let bytes = value.to_le_bytes();
    //     self.0[address..address + 4].copy_from_slice(&bytes);
    // }
}

pub trait Addressable: ReadWrite<u8> {
    fn is_empty(&self) -> bool;
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

impl Addressable for LinearMemory {
    fn is_empty(&self) -> bool {
        self.0[0] == 0
    }
}

#[derive(Debug, Clone, Copy)]
enum Permission {
    R,
    W,
    X,
}

#[derive(Debug, Default)]
struct Permissions([bool; 3]);

impl Permissions {
    fn new(permissions: &[Permission]) -> Permissions {
        let mut perm: [bool; 3] = [false, false, false];
        for p in permissions {
            perm[*p as usize] = true
        }

        Permissions(perm)
    }

    fn enable(&mut self, permission: Permission) {
        self.0[permission as usize] = true;
    }

    fn disable(&mut self, permission: Permission) {
        self.0[permission as usize] = false;
    }
}

// impl Index<Permission> for Permissions {
//     type Output = * bool;

//     fn index(&self, permission: Permission) -> &Self::Output {
//         match permission {
//             Permission::R => self.0[0],
//             Permission::W => self.0[1],
//             Permission::X => self.0[2],
//         }
//     }
// }

#[derive(Debug)]
enum RegionType {
    Code,
    Data,
    Heap,
    Stack,
}

#[derive(Default, Debug)]
struct RegionRange(u32, u32);
impl RegionRange {
    fn new(start: u32, end: u32) -> RegionRange {
        RegionRange(start, end)
    }
}

#[derive(Debug)]
struct Region {
    permissions: Permissions,
    range: RegionRange,
    t: RegionType,
}

impl Region {
    fn new(permissions: Permissions, range: RegionRange, t: RegionType) -> Region {
        Region {
            permissions,
            range,
            t,
        }
    }

    // /// Check if the address is valid
    // pub fn is_valid(&self, address: usize) -> bool {
    //     debug!(
    //         "start: {0}. address: {address}. size: {1}",
    //         self.start, self.size
    //     );

    //     // self.start <= address && address < (self.size + self.start)
    // }

    // pub fn offset(&self, address: usize) -> usize {
    //     address - self.start
    // }
}

// code (0x0000_0000, 0x0000_1000)
// data (0x0000_1000, 0x4000_0000)
// heap (0x4000_0000, 0x8000_0000)
// stack  (0x8000_0000, 0xFFFF_FFFF)
pub struct Regions([Region; 4]);

impl Regions {
    // fn valid(&self) -> i32 {
    //     let f = 0xFFFFF000;
    //     // 4294963200
    //     // 4294967295
    //     let fg= u32::MAX;
    //     1
    // }
}

impl Index<RegionType> for Regions {
    type Output = Region;

    fn index(&self, t: RegionType) -> &Self::Output {
        match t {
            RegionType::Code => &self.0[0],
            RegionType::Data => &self.0[1],
            RegionType::Heap => &self.0[2],
            RegionType::Stack => &self.0[3],
        }
    }
}

impl IndexMut<RegionType> for Regions {
    fn index_mut(&mut self, t: RegionType) -> &mut Self::Output {
        match t {
            RegionType::Code => &mut self.0[0],
            RegionType::Data => &mut self.0[1],
            RegionType::Heap => &mut self.0[2],
            RegionType::Stack => &mut self.0[3],
        }
    }
}

impl Default for Regions {
    fn default() -> Self {
        Self([
            Region::new(
                Permissions::default(),
                RegionRange::default(),
                RegionType::Code,
            ), //(0x0000_0000, 0x0000_1000)
            Region::new(
                Permissions::default(),
                RegionRange::default(),
                RegionType::Data,
            ), //(0x0000_1000, 0x4000_0000)
            Region::new(
                Permissions::default(),
                RegionRange::default(),
                RegionType::Heap,
            ), //(0x4000_0000, 0x8000_0000)
            Region::new(
                Permissions::default(),
                RegionRange::default(),
                RegionType::Stack,
            ), //(0x8000_0000, 0xFFFF_FFFF)
        ])
    }
}

// #[derive(Debug)]
// pub struct LocalAddress {
//     offset: usize,
//     memory_Index: usize,
// }

pub struct MemoryManager<M> {
    memory: M,
    regions: Regions,
}

impl<M> MemoryManager<M>
where
    M: Addressable,
{
    pub fn new(memory: M) -> MemoryManager<M> {
        MemoryManager {
            memory,
            regions: Regions::default(),
        }
    }

    fn load_program(&mut self, program: &[u32], start_address: usize) {
        self.regions[RegionType::Code]
            .permissions
            .enable(Permission::W);
        let end_address = start_address + program.len();
        // TODO: FIX ME
        self.regions[RegionType::Code]
            .permissions
            .disable(Permission::W);
        // self.0[start_address..end_address].copy_from_slice(program);
    }

    #[cfg(test)]
    pub fn load_program_test<T>(&mut self, program: &[T], addr: u32)
    where
        M: ReadWrite<T>,
        T: Copy,
    {
        self.regions[RegionType::Code].range = RegionRange::new(0, 0x0000_1000);
        self.regions[RegionType::Code]
            .permissions
            .enable(Permission::W);

        for (i, b) in program.iter().enumerate() {
            let addr = addr + ((i as u32) * 4);
            self.memory.write(addr as usize, *b).unwrap();
        }

        self.regions[RegionType::Code]
            .permissions
            .disable(Permission::W);
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

        let r = &self.regions[RegionType::Code].range;
        println!("Code segment start: {}, end: {}", r.0, r.1);
        // Code segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            let sz = self.memory.is_empty();
            println!("Size: {sz}");
            if is_write && !sz {
                return Err("Write to code segment");
            }
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Data].range;
        println!("Data segment start: {}, end: {}", r.0, r.1);
        // Data segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            if is_write {
                return Err("Write to data segment");
            }
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Heap].range;
        println!("Heap segment start: {}, end: {}", r.0, r.1);
        // Heap segment (read/write)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Stack].range;
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
    //     self.regions.push(Region::new(start, size, memory));,
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
