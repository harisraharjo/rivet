use shared::EnumCount;

#[derive(EnumCount, Default, Debug)]
pub enum SectionType {
    #[default]
    Text,
    Data,
    Rodata,
    Bss,
}

pub struct Section {
    ty: SectionType,
    lc: usize,
}

// pub struct Sections {
//     buffer: [Section; SectionType::VARIANT_COUNT],
//     active: SectionType,
// }
