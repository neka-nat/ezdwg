use crate::bit::BitReader;
use crate::core::result::Result;
use crate::entities::dim_linear::{decode_dim_linear, DimLinearEntity};

pub type DimRadiusEntity = DimLinearEntity;

pub fn decode_dim_radius(reader: &mut BitReader<'_>) -> Result<DimRadiusEntity> {
    // R2000/R2004 radius dimensions share a largely compatible body layout
    // with linear dimensions for the fields we currently surface.
    decode_dim_linear(reader)
}
