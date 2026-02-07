use crate::bit::BitReader;
use crate::core::result::Result;
use crate::entities::dim_linear::{decode_dim_linear, DimLinearEntity};

pub type DimDiameterEntity = DimLinearEntity;

pub fn decode_dim_diameter(reader: &mut BitReader<'_>) -> Result<DimDiameterEntity> {
    // R2000/R2004 diameter dimensions share a largely compatible body layout
    // with linear dimensions for the fields we currently surface.
    decode_dim_linear(reader)
}
