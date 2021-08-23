use crate::bit::BitReader;
use crate::core::error::{DwgError, ErrorKind};
use crate::core::result::Result;

#[derive(Debug, Clone, Copy)]
pub struct ObjectRecord<'a> {
    pub offset: u32,
    pub size: u32,
    pub body_start: usize,
    pub body_bit_pos: u8,
    pub body: &'a [u8],
}

impl<'a> ObjectRecord<'a> {
    pub fn body_range(&self) -> (usize, usize) {
        let start = self.body_start;
        let end = start + self.size as usize;
        (start, end)
    }

    pub fn record_range(&self) -> (usize, usize) {
        let start = self.offset as usize;
        let end = self.body_start + self.size as usize + 2;
        (start, end)
    }

    pub fn bit_reader(&self) -> BitReader<'a> {
        let mut reader = BitReader::new(self.body);
        reader.set_pos(0, self.body_bit_pos);
        reader
    }
}

pub fn parse_object_record<'a>(bytes: &'a [u8], offset: u32) -> Result<ObjectRecord<'a>> {
    let offset_usize = offset as usize;
    if offset_usize >= bytes.len() {
        return Err(
            DwgError::new(ErrorKind::Format, "object record offset exceeds file size")
                .with_offset(offset as u64),
        );
    }

    let mut reader = BitReader::new(bytes);
    reader.set_pos(offset_usize, 0);

    let size = reader.read_ms()?; // size in bytes excluding CRC
    if size == 0 {
        return Err(
            DwgError::new(ErrorKind::Format, "object record size is zero")
                .with_offset(offset as u64),
        );
    }

    let (body_start, body_bit_pos) = reader.get_pos();
    let end = body_start
        .checked_add(size as usize)
        .ok_or_else(|| DwgError::new(ErrorKind::Format, "object size overflow"))?;
    if end + 2 > bytes.len() {
        return Err(DwgError::new(
            ErrorKind::Format,
            format!("object record exceeds file size: end {end} + crc"),
        )
        .with_offset(offset as u64));
    }

    let body = &bytes[body_start..end];

    Ok(ObjectRecord {
        offset,
        size,
        body_start,
        body_bit_pos,
        body,
    })
}
