use std::ops::{Add, Range};

pub use macros_derive::{EnumCount, EnumVariants, VMInstruction};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Unknown Opcode: `{0}`")]
    UnknownOpcode(u8),
}

pub trait VMInstruction {
    fn opcode(&self) -> u8;
    // fn instruction_format(&self, bytes: u32) -> i32;
}

pub trait EnumCount {
    const VARIANT_COUNT: usize;
}

pub trait EnumVariants<const N: usize> {
    fn variants<'a>() -> [&'a str; N];
}

pub const fn max_value_for_bit_length<const SIGNED: bool>(bit_count: u32) -> u32 {
    if !SIGNED {
        (1u32 << bit_count) - 1
    } else {
        (1 << (bit_count - 1)) - 1
    }
}

/// An iterator over non-overlapping chunks of a Range<usize>.
pub struct Chunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    range: Range<T>,
    size: usize,
    current: T,
}

impl<T> Chunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    fn new(range: Range<T>, size: usize) -> Self {
        let current = range.start;
        Self {
            current,
            range,
            size,
        }
    }
}

impl<T> Iterator for Chunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    type Item = Range<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.range.end {
            return None;
        }

        let start = self.current;
        let end = (start + self.size).min(self.range.end); // Exclusive upper bound
        self.current = end; // Move to next chunk
        Some(start..end)
    }
}

pub trait RangeChunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    fn chunks(self, size: usize) -> Chunks<T>;
}

impl RangeChunks<usize> for Range<usize> {
    fn chunks(self, size: usize) -> Chunks<usize> {
        Chunks::new(self, size)
    }
}
