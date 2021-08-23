use crate::bit::BitReader;
use crate::container::section_directory::SectionLocatorRecord;
use crate::container::section_loader::SectionSlice;
use crate::io::ByteReader;

#[derive(Debug, Clone, Copy)]
pub struct StreamView<'a> {
    section: SectionSlice<'a>,
}

impl<'a> StreamView<'a> {
    pub fn new(section: SectionSlice<'a>) -> Self {
        Self { section }
    }

    pub fn record(&self) -> SectionLocatorRecord {
        self.section.record
    }

    pub fn offset(&self) -> u32 {
        self.section.record.offset
    }

    pub fn size(&self) -> u32 {
        self.section.record.size
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        self.section.data
    }

    pub fn byte_reader(&self) -> ByteReader<'a> {
        ByteReader::new(self.section.data)
    }

    pub fn bit_reader(&self) -> BitReader<'a> {
        BitReader::new(self.section.data)
    }
}
