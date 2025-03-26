use std::ops::Range;

pub(crate) struct Span {
    start: u8,
    end: u8,
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        (value.start as usize)..(value.end as usize)
    }
}
