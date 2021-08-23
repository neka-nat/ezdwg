use crate::bit::{BitReader, Endian};
use crate::core::result::Result;
use crate::entities::common::parse_common_entity_header;

#[derive(Debug, Clone)]
pub struct InsertEntity {
    pub handle: u64,
    pub position: (f64, f64, f64),
    pub scale: (f64, f64, f64),
    pub rotation: f64,
}

pub fn decode_insert(reader: &mut BitReader<'_>) -> Result<InsertEntity> {
    let header = parse_common_entity_header(reader)?;

    let position = reader.read_3bd()?;
    let data_flags = reader.read_bb()?;

    let (x_scale, y_scale, z_scale) = match data_flags {
        0x03 => (1.0, 1.0, 1.0),
        0x01 => {
            let y = reader.read_dd(1.0)?;
            let z = reader.read_dd(1.0)?;
            (1.0, y, z)
        }
        0x02 => {
            let x = reader.read_rd(Endian::Little)?;
            (x, x, x)
        }
        _ => {
            let x = reader.read_rd(Endian::Little)?;
            let y = reader.read_dd(x)?;
            let z = reader.read_dd(x)?;
            (x, y, z)
        }
    };

    let rotation = reader.read_bd()?;
    let _extrusion = reader.read_3bd()?;
    let has_attribs = reader.read_b()?;
    if has_attribs == 1 {
        let _owned_obj_count = reader.read_bl()?;
    }

    Ok(InsertEntity {
        handle: header.handle,
        position,
        scale: (x_scale, y_scale, z_scale),
        rotation,
    })
}
