use crate::bit::{BitReader, Endian};
use crate::core::result::Result;
use crate::entities::common::parse_common_entity_header;

#[derive(Debug, Clone)]
pub struct MInsertEntity {
    pub handle: u64,
    pub position: (f64, f64, f64),
    pub scale: (f64, f64, f64),
    pub rotation: f64,
    pub num_columns: u16,
    pub num_rows: u16,
    pub column_spacing: f64,
    pub row_spacing: f64,
}

pub fn decode_minsert(reader: &mut BitReader<'_>) -> Result<MInsertEntity> {
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

    let num_columns = reader.read_bs()?;
    let num_rows = reader.read_bs()?;
    let column_spacing = reader.read_bd()?;
    let row_spacing = reader.read_bd()?;

    Ok(MInsertEntity {
        handle: header.handle,
        position,
        scale: (x_scale, y_scale, z_scale),
        rotation,
        num_columns,
        num_rows,
        column_spacing,
        row_spacing,
    })
}
