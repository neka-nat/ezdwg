use crate::bit::BitReader;
use crate::core::result::Result;
use crate::entities::common::parse_common_entity_header;

#[derive(Debug, Clone)]
pub struct EllipseEntity {
    pub handle: u64,
    pub center: (f64, f64, f64),
    pub major_axis: (f64, f64, f64),
    pub extrusion: (f64, f64, f64),
    pub axis_ratio: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

pub fn decode_ellipse(reader: &mut BitReader<'_>) -> Result<EllipseEntity> {
    let header = parse_common_entity_header(reader)?;

    let center = reader.read_3bd()?;
    let major_axis = reader.read_3bd()?;
    let extrusion = reader.read_3bd()?;
    let axis_ratio = reader.read_bd()?;
    let start_angle = reader.read_bd()?;
    let end_angle = reader.read_bd()?;

    Ok(EllipseEntity {
        handle: header.handle,
        center,
        major_axis,
        extrusion,
        axis_ratio,
        start_angle,
        end_angle,
    })
}
