use geoarrow::array::{CoordType, WKBArray};
use geoarrow::datatypes::{Dimension, SerializedType};
use geoarrow::io::geozero::FromEWKB;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_arrow::PyArray;

use crate::ffi::to_python::native_array_to_pyobject;
use pyo3_geoarrow::PyGeoArrowResult;

#[pyfunction]
pub fn from_ewkb(py: Python, input: PyArray) -> PyGeoArrowResult<PyObject> {
    let (array, field) = input.into_inner();
    let typ = SerializedType::try_from(field.as_ref())?;
    let geo_array = match typ {
        SerializedType::WKB => {
            let wkb_arr = WKBArray::<i32>::try_from((array.as_ref(), field.as_ref()))?;
            FromEWKB::from_ewkb(
                &wkb_arr,
                CoordType::Interleaved,
                Dimension::XY,
                Default::default(),
                false,
            )?
        }
        SerializedType::LargeWKB => {
            let wkb_arr = WKBArray::<i64>::try_from((array.as_ref(), field.as_ref()))?;
            FromEWKB::from_ewkb(
                &wkb_arr,
                CoordType::Interleaved,
                Dimension::XY,
                Default::default(),
                false,
            )?
        }
        _ => return Err(PyValueError::new_err("Expected a WKB array").into()),
    };
    native_array_to_pyobject(py, geo_array)
}
