use crate::array::primitive::BooleanArray;
use crate::array::CoordBuffer;
use crate::array::GeometryArray;
use crate::broadcasting::{BroadcastableAffine, BroadcastableFloat};
use crate::error::WasmResult;
use crate::impl_geometry_array;
use crate::log;
#[cfg(feature = "geodesy")]
use crate::reproject::ReprojectDirection;
use crate::utils::vec_to_offsets;
use crate::TransformOrigin;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LineStringArray(pub(crate) geoarrow::array::LineStringArray);

impl_geometry_array!(LineStringArray);

#[wasm_bindgen]
impl LineStringArray {
    #[wasm_bindgen(constructor)]
    pub fn new(
        coords: CoordBuffer,
        geom_offsets: Vec<i32>,
        validity: Option<BooleanArray>,
    ) -> Self {
        Self(geoarrow::array::LineStringArray::new(
            coords.0,
            vec_to_offsets(geom_offsets),
            validity.map(|validity| validity.0.values().clone()),
        ))
    }
}

impl From<&LineStringArray> for geoarrow::array::GeometryArray {
    fn from(value: &LineStringArray) -> Self {
        geoarrow::array::GeometryArray::LineString(value.0.clone())
    }
}

impl From<geoarrow::array::LineStringArray> for LineStringArray {
    fn from(value: geoarrow::array::LineStringArray) -> Self {
        Self(value)
    }
}
