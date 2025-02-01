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
