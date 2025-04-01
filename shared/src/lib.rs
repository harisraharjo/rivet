use std::ops::{Add, Neg, Range, RangeInclusive, Sub};

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
pub struct RangeChunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    constant: usize,
    current: T,
    end: T,
}

impl<T> RangeChunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T> + Sub<usize, Output = T>,
{
    fn new(range: Range<T>, size: usize) -> Self {
        let current = range.start;
        Self {
            current,
            end: range.end - 1,
            constant: size - 1,
        }
    }
}

impl<T> Iterator for RangeChunks<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let start = self.current;
        let end = (start + self.constant).min(self.end); // Exclusive upper bound
        self.current = end + 1; // Move to next chunk
        Some(start..=end)
    }
}

pub trait ChunksExt<T>
where
    T: Copy + PartialOrd + Ord + Add<usize, Output = T>,
{
    fn chunks(self, size: usize) -> RangeChunks<T>;
}

impl ChunksExt<usize> for Range<usize> {
    fn chunks(self, size: usize) -> RangeChunks<usize> {
        RangeChunks::new(self, size)
    }
}
