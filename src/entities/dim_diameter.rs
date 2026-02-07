use crate::bit::BitReader;
use crate::core::result::Result;
use crate::entities::dim_linear::{
    decode_dim_linear, decode_dim_linear_r2007, decode_dim_linear_r2010, decode_dim_linear_r2013,
    DimLinearEntity,
};

pub type DimDiameterEntity = DimLinearEntity;

pub fn decode_dim_diameter(reader: &mut BitReader<'_>) -> Result<DimDiameterEntity> {
    // R2000/R2004 diameter dimensions share a largely compatible body layout
    // with linear dimensions for the fields we currently surface.
    decode_dim_linear(reader)
}

pub fn decode_dim_diameter_r2007(reader: &mut BitReader<'_>) -> Result<DimDiameterEntity> {
    decode_dim_linear_r2007(reader)
}

pub fn decode_dim_diameter_r2010(
    reader: &mut BitReader<'_>,
    object_data_end_bit: u32,
) -> Result<DimDiameterEntity> {
    decode_dim_linear_r2010(reader, object_data_end_bit)
}

pub fn decode_dim_diameter_r2013(
    reader: &mut BitReader<'_>,
    object_data_end_bit: u32,
) -> Result<DimDiameterEntity> {
    decode_dim_linear_r2013(reader, object_data_end_bit)
}
