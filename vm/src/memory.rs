// use std::{marker::PhantomData, ops::Range};

use std::{
    fmt::Debug,
    ops::{Index, IndexMut, Range},
};

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

pub trait ReadWrite<T>
where
    T: Copy,
{
    fn read(&self, address: usize) -> Result<T, MemoryError>;
    fn write(&mut self, address: usize, value: T) -> Result<(), MemoryError>;
    // fn bulk_write(&self, address: usize, value: T) -> i32 {
    //     let bytes = value.to_le_bytes();
    //     self.0[address..address + 4].copy_from_slice(&bytes);
    // }
}

pub trait Addressable: ReadWrite<u8> + ReadWrite<u32> + Index<Range<usize>>
where
    Self::Output: Debug,
{
    fn is_empty(&self) -> bool;
}

#[derive(Debug)]
pub struct LinearMemory(Vec<u8>);
impl LinearMemory {
    pub fn new(size: usize) -> LinearMemory {
        LinearMemory(vec![0; size])
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    // #[inline(always)]
    // fn bulk_write<const BYTES: usize>(&mut self, address: usize, value: &[u8]) {
    //     self.bulk_write::<2>(address, &value.to_le_bytes());
    //     self.0[address..address + BYTES].copy_from_slice(value);
    // }
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

impl Index<Range<usize>> for LinearMemory {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
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

    fn status(&self, permission: Permission) -> bool {
        self.0[permission as usize]
    }
}

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
        RegionRange(start, end + 1)
    }

    fn start(&self) -> u32 {
        self.0
    }

    /// Exclusive
    fn end(&self) -> u32 {
        self.1
    }
}

impl From<&RegionRange> for Range<usize> {
    fn from(value: &RegionRange) -> Self {
        Self {
            start: value.0 as usize,
            end: value.1 as usize,
        }
    }
}

#[derive(Debug)]
struct Region {
    permissions: Permissions,
    range: RegionRange,
    t: RegionType,
    // size: usize
}

impl Region {
    fn new(permissions: Permissions, range: RegionRange, t: RegionType) -> Region {
        Region {
            permissions,
            range,
            t,
            // size,
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

pub struct MemoryManager {
    memory: LinearMemory,
    regions: Regions,
}

impl MemoryManager {
    pub fn new(memory_allocation: usize) -> MemoryManager {
        MemoryManager {
            memory: LinearMemory::new(memory_allocation),
            regions: Regions::default(),
        }
    }

    pub fn load_program(&mut self, program: &[u8], start_address: u32) -> Result<(), MemoryError> {
        // rounding up to the nearest alignment in case the program length is not aligned. //TODO: Decide giving region memory alignment or not
        let alignment = 4;
        let end = (program.len() as u32).div_ceil(alignment) * alignment;

        self.regions[RegionType::Code].range = RegionRange::new(0, end);
        self.regions[RegionType::Code]
            .permissions
            .enable(Permission::W);

        let mut current_address = start_address;
        // Handle full 4-byte chunks
        for chunk in program.chunks(4) {
            if chunk.len() == 4 {
                // Write full 4 bytes
                let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                self.write::<u32>(current_address, word)?;
                current_address += 4;
            } else {
                // TODO: Decide this with padding or not
                panic!("CHUNK IS NOT 4 BYTES");
                // Handle the remaining bytes which are less than 4
                // for &byte in chunk {
                //     self.write_byte(current_address, byte);
                // current_address += 1;
                // }
            }
        }

        // TODO: FIX ME
        self.regions[RegionType::Code]
            .permissions
            .disable(Permission::W);

        Ok(())
    }

    /// Validate address and alignment, return buffer offset
    fn validate(
        &self,
        vaddr: u32,
        size: u32, // 1, 2, or 4 bytes
        is_write: bool,
    ) -> Result<usize, &'static str> {
        // Alignment check (RISC-V requires alignment for LW/SW/LH/SH)
        if vaddr % size != 0 {
            return Err("Unaligned access");
        }

        let r = &self.regions[RegionType::Code].range;
        // Code segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            let is_immutable: bool = self.regions[RegionType::Code]
                .permissions
                .status(Permission::W);

            if is_write && !is_immutable {
                return Err("Write to code segment");
            }
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Data].range;
        // Data segment (read-only)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            if is_write {
                return Err("Write to data segment");
            }
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Heap].range;
        // Heap segment (read/write)
        if vaddr >= r.0 && vaddr + size <= r.1 {
            return Ok(vaddr as usize);
        }

        let r = &self.regions[RegionType::Stack].range;
        // Stack segment (read/write, grows downward)
        if vaddr <= r.0 && vaddr >= r.1 {
            return Ok(vaddr as usize);
        }

        Err("Address out of bounds")
    }

    pub fn read<T>(&self, address: u32) -> Result<T, MemoryError>
    where
        T: Copy,
        LinearMemory: ReadWrite<T>,
    {
        let a = &self.regions[RegionType::Code].range;
        let data = &self.memory[a.into()];

        // TODO: TIDY ME.
        let real_addr = self.validate(address, 4, false).unwrap();
        self.memory.read(real_addr)
    }

    pub fn write<T>(&mut self, address: u32, value: T) -> Result<(), MemoryError>
    where
        T: Copy,
        LinearMemory: ReadWrite<T>,
    {
        let real_addr = self.validate(address, 4, true).unwrap();
        let a = &self.regions[RegionType::Code].range;
        let data = &self.memory[a.into()];

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
