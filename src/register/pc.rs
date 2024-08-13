#[derive(Default, Debug)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    pub const fn new() -> ProgramCounter {
        ProgramCounter(0)
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 1
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        self.0
    }
}
