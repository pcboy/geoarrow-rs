use std::sync::Arc;

use arrow::datatypes::DataType;
use arrow_array::cast::AsArray;
use geoarrow::array::CoordType;
use geoarrow::io::geozero::FromWKT;
use geoarrow::GeometryArrayTrait;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3_arrow::PyArray;

use crate::array::*;
use crate::error::PyGeoArrowResult;
use crate::ffi::to_python::geometry_array_to_pyobject;

/// Parse an Arrow StringArray from WKT to its GeoArrow-native counterpart.
///
/// Args:
///     input: An Arrow array of string type holding WKT-formatted geometries.
///
/// Returns:
///     A GeoArrow-native geometry array
#[pyfunction]
pub fn from_wkt(input: PyArray) -> PyGeoArrowResult<PyObject> {
    let (array, _field) = input.into_inner();
    let geo_array: Arc<dyn GeometryArrayTrait> = match array.data_type() {
        DataType::Utf8 => FromWKT::from_wkt(
            array.as_string::<i32>(),
            CoordType::Interleaved,
            Default::default(),
            false,
        )?,
        DataType::LargeUtf8 => FromWKT::from_wkt(
            array.as_string::<i64>(),
            CoordType::Interleaved,
            Default::default(),
            false,
        )?,
        other => {
            return Err(PyTypeError::new_err(format!("Unexpected array type {:?}", other)).into())
        }
    };
    Python::with_gil(|py| geometry_array_to_pyobject(py, geo_array))
}

macro_rules! impl_from_wkt {
    ($py_array:ty, $geoarrow_array:ty) => {
        #[pymethods]
        impl $py_array {
            /// Parse an Arrow StringArray from WKT to its GeoArrow-native counterpart.
            ///
            /// Args:
            ///     input: An Arrow array of string type holding WKT-formatted geometries.
            ///
            /// Returns:
            ///     A GeoArrow-native geometry array
            #[classmethod]
            pub fn from_wkt(_cls: &Bound<PyType>, input: PyArray) -> PyGeoArrowResult<$py_array> {
                let (array, _field) = input.into_inner();
                match array.data_type() {
                    DataType::Utf8 => Ok(<$geoarrow_array>::from_wkt(
                        array.as_string::<i32>(),
                        CoordType::Interleaved,
                        Default::default(),
                        false,
                    )?
                    .into()),
                    DataType::LargeUtf8 => Ok(<$geoarrow_array>::from_wkt(
                        array.as_string::<i64>(),
                        CoordType::Interleaved,
                        Default::default(),
                        false,
                    )?
                    .into()),
                    other => Err(PyTypeError::new_err(format!(
                        "Unexpected array type {:?}",
                        other
                    ))
                    .into()),
                }
            }
        }
    };
}

impl_from_wkt!(MixedGeometryArray, geoarrow::array::MixedGeometryArray<i32, 2>);
impl_from_wkt!(
    GeometryCollectionArray,
    geoarrow::array::GeometryCollectionArray<i32, 2>
);
