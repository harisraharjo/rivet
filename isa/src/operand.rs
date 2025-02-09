use crate::instruction::Codec;
use std::ops::{BitAnd, Shl};

pub(crate) enum ImmediateType {
    B14,
    B19,
}

impl ImmediateType {
    pub const fn length(&self) -> u32 {
        match self {
            ImmediateType::B14 => 14,
            ImmediateType::B19 => 19,
        }
    }
}

impl From<ImmediateType> for u32 {
    fn from(value: ImmediateType) -> Self {
        value.length()
    }
}

// TODO: Make generic immediate or make it u32 instead
#[derive(Debug, PartialEq, Eq)]
pub struct Immediate<const BIT: u32>(i32);
pub type Immediate14 = Immediate<14>;
pub type Immediate19 = Immediate<19>;

impl<const BIT: u32> Immediate<BIT> {
    const _BIT: () = if BIT == 14 || BIT == 19 {
        ()
    } else {
        panic!("Immediate only support 14 & 19 Bit");
    };

    pub fn new(value: i32) -> Self {
        let _ = Self::_BIT;

        let max = (1 << (BIT - 1)) - 1; //i32
                                        // let max = (1 << BIT) - 1; //u32
        let min = -(max + 1);
        assert!(
            value >= min && value <= max,
            "the value does not fit into the type `Immediate`"
        );

        Immediate(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    fn convert_twos_complement_to_i32(masked_value: u32, bit_mask: u32) -> i32 {
        // Check if the sign bit is set

        let sign_bit = 1u32 << (bit_mask.trailing_ones() - 1);

        if masked_value & sign_bit != 0 {
            // Negative number: extend sign by converting to two's complement
            let positive_counterpart = (sign_bit << 1) - masked_value;

            -(positive_counterpart as i32)

            // Self(-(positive_counterpart as i32))
        } else {
            masked_value as i32
            // Positive number or zero
            // Self(masked_value as i32)
        }
    }
}

impl Immediate<{ ImmediateType::B14.length() }> {}
impl Immediate<{ ImmediateType::B19.length() }> {}

impl<const BIT: u32> Codec for Immediate<BIT> {
    fn decode(src: u32, bit_accumulation: u32, bit_mask: u32) -> Self
    where
        Self: From<u32>,
    {
        let _ = Self::_BIT;
        Self(Self::convert_twos_complement_to_i32(
            (src >> bit_accumulation) & bit_mask,
            bit_mask,
        ))
        .into()
    }
}

impl<const BIT: u32> From<Immediate<BIT>> for i32 {
    fn from(value: Immediate<BIT>) -> Self {
        value.0
    }
}

impl<const BIT: u32> From<u32> for Immediate<BIT> {
    fn from(value: u32) -> Self {
        Immediate(value as i32)
    }
}

impl<const BIT: u32> From<Immediate<BIT>> for u32 {
    fn from(value: Immediate<BIT>) -> Self {
        // let g = value.0;
        // println!("Immi: {:032b}", g);
        // println!("Immi: {:032b}", g as u32);
        value.0 as u32
    }
}

impl<const BIT: u32> BitAnd<u32> for &Immediate<BIT> {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (self.0 as u32) & rhs
    }
}

impl<const BIT: u32> Shl<u32> for &Immediate<BIT> {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self.0 as u32) << rhs
    }
}

#[cfg(test)]
mod test_super {
    use super::*;

    // TODO: add test for overflow
    // Immediate14::new(-0x1FFF),
    //  assert_eq!(
    //         vm.registers().get(Register::A0),
    //         Immediate14::new(-0x1FFF).into(),
    //         "Addi error"
    //     );

    //     assert_eq!(
    //         vm.registers().get(Register::T0),
    //         Immediate19::new(0x3FFFF).into(),
    //         "lui error"
    //     );

    #[test]
    fn t_imm() {
        let imm = Immediate14::new(-31);
        let result = imm.encode(0x3FFF, 18);
        assert_eq!(result, 0xFF840000);

        let result = Immediate::decode(4286958867, 18, 0x3FFF);
        assert_eq!(result, imm);
    }

    // #[test]
    // #[should_panic]
    // fn t_imm_panic() {
    //     let result = std::panic::catch_unwind(|| Immediate::<32>::new(-0x1FFF));
    //     assert!(result.is_err());
    // }
}

// use std::collections::HashMap;
// #[derive(Debug, Clone, Copy)]
// enum Permissions {
//     Read = 1,
//     Write = 2,
//     Execute = 4,
// }
// #[derive(Debug, Clone)]
// struct MemorySegment {
//     base_address: u32,
//     size: usize,
//     permissions: Permissions,
//     data: Vec<u8>,
// } // VM structure with explicit 32-bit little-endian memory model
// struct VM {
//     memory_segments: HashMap<u32, MemorySegment>,
// }
// impl VM {
//     fn new() -> Self {
//         VM {
//             memory_segments: HashMap::new(),
//         }
//     } // Helper function to write 32-bit word in little-endian

//     fn write_word_le(&mut self, address: u32, value: u32) -> Result<(), String> {
//         let segment = self
//             .memory_segments
//             .get_mut(&(address & 0xFFFFF000))
//             .ok_or("Segment not found")?;

//         if !segment.permissions.contains(Permissions::Write) {
//             return Err("Write permission not granted".to_string());
//         }
//         let offset = (address & 0xFFF) as usize;
//         if offset + 4 > segment.size {
//             return Err("Write would exceed segment boundaries".to_string());
//         }
//         segment.data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
//         Ok(())
//     } // Load program, ensuring we use little-endian for instructions

//     fn load_program(&mut self, program: &[u32]) -> Result<(), String> {
//         let code_segment = self.memory_segments.entry(0x1000).or_insert(MemorySegment {
//             base_address: 0x1000,
//             size: 4096, // Example size, 4KB
//             permissions: Permissions::Read | Permissions::Execute,
//             data: vec![0; 4096],
//         });

//         // Temporarily elevate permissions to write
//         let old_permissions = code_segment.permissions;
//         code_segment.permissions = Permissions::Read | Permissions::Write | Permissions::Execute;

//         for (i, &instruction) in program.iter().enumerate() {
//             let address = 0x1000 + (i * 4) as u32;
//             self.write_word_le(address, instruction)?;
//         }

//         // Revert permissions
//         code_segment.permissions = old_permissions;
//         Ok(())
//     } // Print memory state, showing 32-bit values in little-endian

//     fn print_memory_state(&self) {
//         for (addr, segment) in &self.memory_segments {
//             println!(
//                 "Segment at {:#x}: Size={}, Permissions={:?}",
//                 addr, segment.size, segment.permissions
//             );
//             for i in (0..segment.size).step_by(4).take(10) {
//                 // Print first 10 words
//                 if i + 4 <= segment.size {
//                     let word = u32::from_le_bytes(segment.data[i..i + 4].try_into().unwrap());
//                     println!(" Address {:#x}: {:#010X}", addr + i as u32, word);
//                 }
//             }
//         }
//     }
// }
// fn main() {
//     let mut vm = VM::new(); // Dummy program with some placeholder 32-bit instructions in little-endian
//     let dummy_program: [u32; 4] = [0x01020304, 0x05060708, 0x090A0B0C, 0x0D0E0F10];
//     match vm.load_program(&dummy_program) {
//         Ok(_) => println!("Program loaded successfully"),
//         Err(e) => println!("Failed to load program: {}", e),
//     }
//     vm.print_memory_state();
// }
