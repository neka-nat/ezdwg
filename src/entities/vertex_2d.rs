use crate::bit::{BitReader, Endian};
use crate::core::result::Result;
use crate::entities::common::{parse_common_entity_handles, parse_common_entity_header};

#[derive(Debug, Clone)]
pub struct Vertex2dEntity {
    pub handle: u64,
    pub flags: u16,
    pub position: (f64, f64, f64),
    pub start_width: f64,
    pub end_width: f64,
    pub bulge: f64,
    pub tangent_dir: f64,
}

pub fn decode_vertex_2d(reader: &mut BitReader<'_>) -> Result<Vertex2dEntity> {
    let header = parse_common_entity_header(reader)?;
    // Flags are NOT bit-pair-coded in the DWG spec for VERTEX(2D).
    let flags = reader.read_rs(Endian::Little)?;
    let position = reader.read_3bd()?;

    let mut start_width = reader.read_bd()?;
    let end_width = if start_width < 0.0 {
        start_width = -start_width;
        start_width
    } else {
        reader.read_bd()?
    };

    let bulge = reader.read_bd()?;
    let tangent_dir = reader.read_bd()?;

    let _handles = parse_common_entity_handles(reader, &header)?;

    Ok(Vertex2dEntity {
        handle: header.handle,
        flags,
        position,
        start_width,
        end_width,
        bulge,
        tangent_dir,
    })
}
