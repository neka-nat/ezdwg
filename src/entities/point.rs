use crate::bit::BitReader;
use crate::core::result::Result;
use crate::entities::common::parse_common_entity_header;

#[derive(Debug, Clone)]
pub struct PointEntity {
    pub handle: u64,
    pub location: (f64, f64, f64),
    pub x_axis_angle: f64,
}

pub fn decode_point(reader: &mut BitReader<'_>) -> Result<PointEntity> {
    let header = parse_common_entity_header(reader)?;

    let location = reader.read_3bd()?;
    let _thickness = reader.read_bt()?;
    let _extrusion = reader.read_be()?;
    let x_axis_angle = reader.read_bd()?;

    Ok(PointEntity {
        handle: header.handle,
        location,
        x_axis_angle,
    })
}
