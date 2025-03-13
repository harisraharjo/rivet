use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut, Range},
};

use shared::EnumCount;
use thiserror::Error;

// use log::debug;

#[derive(Debug, Error, PartialEq)]
pub enum MemoryError {
    #[error("Permission Denied: Unable to `{0}` at address `{1}`")]
    PermissionDenied(Permission, u32),
    #[error("Unaligned access: Trying to access address `{0}` with `{1}`")]
    UnalignedAccess(u32, usize),
    #[error("Invalid address: `{0}`")]
    InvalidAddress(u32),
    #[error("out of bounds: `{0}`")]
    OutOfBounds(u32),
    #[error("Out of memory: maximum capacity is`{0}`")]
    OutOfMemory(u32),
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

    // #[inline(always)]
    // fn bulk_writes<const BYTES: u32>(&mut self, address: u32, value: &[u8]) where Self: Index<u32> + IndexMut<u32>{
    //     self[address..address + BYTES].copy_from_slice(value);
    // }
}

pub trait Addressable: ReadWrite<u8> + ReadWrite<u32> + Index<Range<usize>>
where
    Self::Output: Debug,
{
}

#[derive(Debug)]
pub struct LinearMemory {
    buffer: Vec<u8>,
    //TODO: Since the VM will be converted to WASM, I personally don't see the benefit of Vec<MaybeUninit<u8>> because essentially "uninitialized" data is in fact initialized with 0 value in wasm
    // buffer: Vec<MaybeUninit<u8>>,
    // size: usize,
}

impl Addressable for LinearMemory {}

impl LinearMemory {
    // TODO: Make sure the capacity doesn't exceed u32::MAX
    pub fn new(size: u32) -> LinearMemory {
        //  assert!(size <= u32::MAX);
        // TODO: normally `vec![0; n]` will use `calloc`. I'm not sure about wasm though, couldn't find any info.\

        // let buffer = Vec::with_capacity(size as usize);

        LinearMemory {
            buffer: vec![0; size as usize],
            // size: size as usize,
        }
    }

    fn zero_all(&mut self) {
        self.buffer.fill(0);
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    fn grow(&mut self, size: u32) -> Result<(), MemoryError> {
        if self.buffer.len() > (u32::MAX as usize) {
            Err(MemoryError::OutOfMemory(u32::MAX))
        } else {
            self.buffer.reserve(size as usize);
            Ok(())
        }
    }

    #[inline(always)]
    fn bulk_writes<const BYTES: usize>(&mut self, address: usize, value: &[u8]) {
        self.buffer[address..address + BYTES].copy_from_slice(value);
    }
}

impl ReadWrite<u8> for LinearMemory {
    fn read(&self, address: usize) -> Result<u8, MemoryError> {
        Ok(self.buffer[address])
    }

    fn write(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        self.buffer[address] = value;
        Ok(())
    }
}

impl ReadWrite<u16> for LinearMemory {
    fn read(&self, address: usize) -> Result<u16, MemoryError> {
        let bytes = &self.buffer[address..address + std::mem::size_of::<u16>()];
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn write(&mut self, address: usize, value: u16) -> Result<(), MemoryError> {
        self.bulk_writes::<{ std::mem::size_of::<u16>() }>(address, &value.to_le_bytes());
        Ok(())
    }
}

impl ReadWrite<u32> for LinearMemory {
    fn read(&self, address: usize) -> Result<u32, MemoryError> {
        // TODO: Write done but Read not yet updated
        let bytes = &self.buffer[address..address + std::mem::size_of::<u32>()];

        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn write(&mut self, address: usize, value: u32) -> Result<(), MemoryError> {
        self.bulk_writes::<{ std::mem::size_of::<u32>() }>(address, &value.to_le_bytes());
        // let unit = unsafe {
        //     std::slice::from_raw_parts_mut(
        //         self.buffer.as_mut_ptr().add(address) as *mut MaybeUninit<u8>,
        //         self.buffer.capacity(),
        //     )
        // };

        // let bytes_len = v.len();
        // let last_input_addr = address + bytes_len;

        // let should_grow = self.buffer.len() < last_input_addr;
        // // let unit_slice = &mut unit[0..bytes_len];
        // unsafe {
        //     std::ptr::copy_nonoverlapping(v.as_ptr(), unit.as_mut_ptr().cast(), bytes_len);
        //     // std::ptr::copy_nonoverlapping(v.as_ptr(), unit_slice.as_mut_ptr().cast(), bytes_len);

        //     if should_grow {
        //         self.buffer.set_len(last_input_addr);
        //     };
        // }

        // println!(
        //     "\x1b[93mMem b4: {:?}\x1b[0m",
        //     &self.buffer[address..address + 4]
        // );

        // self.bulk_writes::<4>(address, &v);
        // println!(
        //     "\x1b[93mMem after: {:?}\x1b[0m",
        //     &self.buffer[address..address + 4]
        // );

        Ok(())
    }
}

impl Index<Range<usize>> for LinearMemory {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.buffer[index]
    }
}

pub struct MemoryManager {
    memory: LinearMemory,
    regions: Regions,
    free_memory: u32,
}

impl MemoryManager {
    pub fn new(configuration: &MemoryConfiguration) -> MemoryManager {
        let mut regions = Regions::default();
        let stack_start = configuration.allocated_memory - 1;
        regions[RegionType::Stack].set_bounds(
            configuration.allocated_memory - configuration.stack_size,
            stack_start,
        );
        MemoryManager {
            memory: LinearMemory::new(configuration.allocated_memory),
            regions,
            free_memory: configuration.allocated_memory,
        }
    }

    #[cfg(test)]
    pub fn capacity(&self) -> usize {
        self.memory.buffer.capacity()
    }

    pub fn stack_start(&self) -> u32 {
        self.regions[RegionType::Stack].bounds.end()
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<(), MemoryError> {
        // TODO: grow stack and heap and then stack pointer

        // rounding up to the nearest alignment in case the program length is not aligned. //TODO: Decide to give region alignment or not if the program legnth is not aligned
        let alignment = 4;
        let code_end = (program.len() as u32).div_ceil(alignment) * alignment;
        self.regions[RegionType::Code].set_bounds(0, code_end);

        self.regions[RegionType::Code]
            .permissions
            .enable(Permission::W);

        let mut current_address = 0;
        println!("Program: {:?}", program);
        // TODO: use chunks_exact
        // Handle full 4-byte chunks
        for chunk in program.chunks(4) {
            if chunk.len() == 4 {
                println!("Chunk: {:?}", chunk);
                // Write full 4 bytes
                let word = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                self.write::<u32>(current_address, word)?;
                current_address += 4
            } else {
                // TODO: Decide to write/add padding to the bytes or not if the program is not aligned
                panic!("CHUNK IS NOT 4 BYTES");
                // Handle the remaining bytes which are less than 4
                // for &byte in chunk {
                //     self.write_byte(current_address, byte);
                // current_address += 1;
                // }
            }
        }

        self.regions[RegionType::Code]
            .permissions
            .disable(Permission::W);

        self.regions[RegionType::Data].set_bounds(code_end, code_end);
        self.regions[RegionType::Heap].bounds = RegionBounds::new(
            self.regions[RegionType::Data].bounds.start(),
            self.regions[RegionType::Data].bounds.end(),
        );

        Ok(())
    }

    /// Alignment check (RISC-V requires alignment for LW/SW/LH/SH)
    pub fn alignment_check(&self, size: usize, address: u32) -> Result<(), MemoryError> {
        if address % (size as u32) != 0 {
            return Err(MemoryError::UnalignedAccess(address, size));
        };

        Ok(())
    }

    /// Validate address and alignment, return buffer offset
    fn validate(
        &self,
        vaddr: u32,
        size: usize, // 1, 2, or 4 bytes
        permission: Permission,
    ) -> Result<usize, MemoryError> {
        if vaddr > (self.memory.buffer.capacity() as u32) {
            return Err(MemoryError::OutOfBounds(vaddr));
        }

        if let Some(region) = self.regions.get(vaddr) {
            if region.allowed_to(permission) {
                Ok(vaddr as usize)
            } else {
                Err(MemoryError::PermissionDenied(permission, vaddr))
            }
        } else {
            Err(MemoryError::InvalidAddress(vaddr))
        }
    }

    pub fn read<T>(&self, address: u32) -> Result<T, MemoryError>
    where
        T: Copy,
        LinearMemory: ReadWrite<T>,
    {
        let real_addr = self.validate(address, std::mem::size_of::<T>(), Permission::R)?;
        self.memory.read(real_addr)
    }

    pub fn write<T>(&mut self, address: u32, value: T) -> Result<(), MemoryError>
    where
        T: Copy,
        LinearMemory: ReadWrite<T>,
    {
        let real_addr = self.validate(address, std::mem::size_of::<T>(), Permission::W)?;
        self.memory.write(real_addr, value)
    }

    pub fn reset(&mut self) {
        self.memory.zero_all();
        self.regions.reset();
    }
}

#[derive(Debug, Clone, Copy, EnumCount, PartialEq)]
enum Permission {
    R,
    W,
    X,
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stringify = match self {
            Permission::R => "Read",
            Permission::W => "Write",
            Permission::X => "Execute",
        };
        write!(f, "{}", stringify)
    }
}

#[derive(Debug)]
struct Permissions([bool; Permission::VARIANT_COUNT]);

impl Permissions {
    fn new(permissions: &[Permission]) -> Permissions {
        let mut perm = Self::default();
        for p in permissions {
            perm[*p] = true
        }

        Permissions(perm.into())
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

impl From<Permissions> for [bool; Permission::VARIANT_COUNT] {
    fn from(value: Permissions) -> Self {
        // TODO: Fishy. If the Permission::VARIANT_COUNT changes it won't be good
        [value.0[0], value.0[1], value.0[2]]
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self([true, false, false])
    }
}

impl Index<Permission> for Permissions {
    type Output = bool;

    fn index(&self, index: Permission) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Permission> for Permissions {
    fn index_mut(&mut self, index: Permission) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Default, Debug)]
/// (start, end) inclusive
struct RegionBounds(u32, u32);
impl RegionBounds {
    fn new(start: u32, end: u32) -> RegionBounds {
        RegionBounds(start, end)
    }

    fn start(&self) -> u32 {
        self.0
    }

    fn start_mut(&mut self, value: u32) {
        self.0 = value;
    }

    fn end(&self) -> u32 {
        self.1
    }

    fn end_mut(&mut self, value: u32) {
        self.1 = value;
    }
}

impl From<&RegionBounds> for Range<usize> {
    fn from(value: &RegionBounds) -> Self {
        Self {
            start: value.0 as usize,
            end: value.1 as usize,
        }
    }
}

// THE ORDER IS IMPORTANT CAUSE IT WILL BE USED EVERYWHERE ELSE
#[derive(Debug, EnumCount)]
enum RegionType {
    Code,
    Data,
    Heap,
    Stack,
}

impl RegionType {
    const fn default_stack_size() -> u32 {
        2048
    }
}

#[derive(Debug)]
pub struct Region {
    permissions: Permissions,
    bounds: RegionBounds,
    ty: RegionType,
    // size: usize
}

impl Region {
    fn new(permissions: Permissions, bounds: RegionBounds, ty: RegionType) -> Region {
        Region {
            permissions,
            bounds,
            ty,
            // size,
        }
    }

    fn grow(&mut self, offset: u32) {
        self.bounds.end_mut(offset);
    }

    fn is_valid_address(&self, address: u32) -> bool {
        address >= self.bounds.start() && address <= self.bounds.end()
    }

    fn set_bounds(&mut self, start: u32, end: u32) {
        self.bounds = RegionBounds::new(start, end);
    }

    fn reset(&mut self) {
        self.bounds.start_mut(0);
        self.bounds.end_mut(0);
    }

    fn allowed_to(&self, permission: Permission) -> bool {
        self.permissions.status(permission)
    }
    // pub fn offset(&self, address: usize) -> usize {
    //     address - self.start
    // }
}

// code (0x0000_0000, 0x0000_1000)
// data (0x0000_1000, 0x4000_0000)
// heap (0x4000_0000, 0x8000_0000)
// stack  (0x8000_0000, 0xFFFF_FFFF)
pub struct Regions([Region; RegionType::VARIANT_COUNT]);

impl Regions {
    fn get(&self, address: u32) -> Option<&Region> {
        let mut res: Option<&Region> = None;

        for region in self.0.iter() {
            println!("Region {:?} bounds: {:?}", region.ty, region.bounds);
            if region.is_valid_address(address) {
                res = Some(region);
                break;
            }
        }

        res
    }

    fn reset(&mut self) {
        for region in &mut self.0 {
            region.reset();
        }
    }

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
        &self.0[t as usize]
    }
}

impl IndexMut<RegionType> for Regions {
    fn index_mut(&mut self, t: RegionType) -> &mut Self::Output {
        &mut self.0[t as usize]
    }
}

impl Default for Regions {
    fn default() -> Self {
        Self([
            Region::new(
                Permissions::default(),
                RegionBounds::default(),
                RegionType::Code,
            ), //(0x0000_0000, 0x0000_1000)
            Region::new(
                Permissions::default(),
                RegionBounds::default(),
                RegionType::Data,
            ), //(0x0000_1000, 0x4000_0000)
            Region::new(
                Permissions::new(&[Permission::R, Permission::W]),
                RegionBounds::default(),
                RegionType::Heap,
            ), //(0x4000_0000, 0x8000_0000)
            Region::new(
                Permissions::new(&[Permission::R, Permission::W]),
                RegionBounds::default(),
                RegionType::Stack,
            ), //(0x8000_0000, 0xFFFF_FFFF)
        ])
    }
}

pub struct MemoryConfiguration {
    allocated_memory: u32,
    stack_size: u32,
}

impl MemoryConfiguration {
    pub fn new(allocated_memory: u32) -> MemoryConfiguration {
        let stack_size = RegionType::default_stack_size();
        assert!(
            stack_size <= allocated_memory,
            "Stack size {} can't be more than the memory size and vice versa",
            stack_size
        );

        MemoryConfiguration {
            allocated_memory,
            stack_size,
        }
    }

    pub fn set_stack_size(mut self, size: u32) {
        self.stack_size = size;
    }
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

    #[test]
    fn t_memory_error() {
        let mut memnager = MemoryManager::new(&MemoryConfiguration::new(1024 * 1024));
        let mut err = memnager.write(0x8000, 44u32).err();
        assert_eq!(Some(MemoryError::InvalidAddress(0x8000)), err);

        err = memnager
            .alignment_check(std::mem::size_of::<u32>(), 0x7FFF)
            .err();
        #[rustfmt::skip]
        assert_eq!(Some(MemoryError::UnalignedAccess(0x7FFF, std::mem::size_of::<u32>())), err);

        err = memnager.write(0x0, 44u32).err();
        assert_eq!(Some(MemoryError::PermissionDenied(Permission::W, 0x0)), err);

        println!("{:?}", err);
    }
}
