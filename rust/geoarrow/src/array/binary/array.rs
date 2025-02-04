use std::sync::Arc;

use crate::array::binary::WKBCapacity;
use crate::array::metadata::ArrayMetadata;
use crate::array::util::{offsets_buffer_i32_to_i64, offsets_buffer_i64_to_i32};
use crate::array::{CoordType, WKBBuilder};
use crate::datatypes::{NativeType, SerializedType};
use crate::error::{GeoArrowError, Result};
use crate::scalar::WKB;
use geo_traits::GeometryTrait;
// use crate::util::{owned_slice_offsets, owned_slice_validity};
use crate::trait_::{ArrayAccessor, ArrayBase, IntoArrow, SerializedArray};
use arrow::array::AsArray;
use arrow_array::OffsetSizeTrait;
use arrow_array::{Array, BinaryArray, GenericBinaryArray, LargeBinaryArray};
use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field};

/// An immutable array of WKB geometries using GeoArrow's in-memory representation.
///
/// This is semantically equivalent to `Vec<Option<WKB>>` due to the internal validity bitmap.
///
/// This array implements [`SerializedArray`], not [`NativeArray`]. This means that you'll need to
/// parse the `WKBArray` into a native-typed GeoArrow array (such as
/// [`PointArray`][crate::array::PointArray]) before using it for computations.
#[derive(Debug, Clone, PartialEq)]
pub struct WKBArray<O: OffsetSizeTrait> {
    pub(crate) data_type: SerializedType,
    pub(crate) metadata: Arc<ArrayMetadata>,
    pub(crate) array: GenericBinaryArray<O>,
}

// Implement geometry accessors
impl<O: OffsetSizeTrait> WKBArray<O> {
    /// Create a new WKBArray from a BinaryArray
    pub fn new(array: GenericBinaryArray<O>, metadata: Arc<ArrayMetadata>) -> Self {
        let data_type = match O::IS_LARGE {
            true => SerializedType::LargeWKB,
            false => SerializedType::WKB,
        };

        Self {
            data_type,
            metadata,
            array,
        }
    }

    /// Returns true if the array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Infer the minimal NativeType that this WKBArray can be casted to.
    #[allow(dead_code)]
    // TODO: is this obsolete with new from_wkb approach that uses downcasting?
    pub(crate) fn infer_geo_data_type(&self, _coord_type: CoordType) -> Result<NativeType> {
        todo!()
        // use crate::io::wkb::reader::r#type::infer_geometry_type;
        // infer_geometry_type(self.iter().flatten(), coord_type)
    }

    /// The lengths of each buffer contained in this array.
    pub fn buffer_lengths(&self) -> WKBCapacity {
        WKBCapacity::new(
            self.array.offsets().last().unwrap().to_usize().unwrap(),
            self.len(),
        )
    }

    /// The number of bytes occupied by this array.
    pub fn num_bytes(&self) -> usize {
        let validity_len = self.nulls().map(|v| v.buffer().len()).unwrap_or(0);
        validity_len + self.buffer_lengths().num_bytes::<O>()
    }

    pub fn into_inner(self) -> GenericBinaryArray<O> {
        self.array
    }

    /// Slices this [`WKBArray`] in place.
    /// # Panic
    /// This function panics iff `offset + length > self.len()`.
    #[inline]
    pub fn slice(&self, offset: usize, length: usize) -> Self {
        assert!(
            offset + length <= self.len(),
            "offset + length may not exceed length of array"
        );
        Self {
            array: self.array.slice(offset, length),
            data_type: self.data_type,
            metadata: self.metadata(),
        }
    }

    pub fn owned_slice(&self, _offset: usize, _length: usize) -> Self {
        todo!()
        // assert!(
        //     offset + length <= self.len(),
        //     "offset + length may not exceed length of array"
        // );
        // assert!(length >= 1, "length must be at least 1");

        // // Find the start and end of the ring offsets
        // let (start_idx, _) = self.array.offsets().start_end(offset);
        // let (_, end_idx) = self.array.offsets().start_end(offset + length - 1);

        // let new_offsets = owned_slice_offsets(self.array.offsets(), offset, length);

        // let mut values = self.array.slice(start_idx, end_idx - start_idx);

        // let validity = owned_slice_validity(self.array.nulls(), offset, length);

        // Self::new(GenericBinaryArray::new(
        //     new_offsets,
        //     values.as_slice().to_vec().into(),
        //     validity,
        // ))
    }

    pub fn with_metadata(&self, metadata: Arc<ArrayMetadata>) -> Self {
        let mut arr = self.clone();
        arr.metadata = metadata;
        arr
    }
}

impl<O: OffsetSizeTrait> ArrayBase for WKBArray<O> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn storage_type(&self) -> DataType {
        self.data_type.to_data_type()
    }

    fn extension_field(&self) -> Arc<Field> {
        self.data_type
            .to_field_with_metadata("geometry", true, &self.metadata)
            .into()
    }

    fn extension_name(&self) -> &str {
        self.data_type.extension_name()
    }

    fn into_array_ref(self) -> Arc<dyn Array> {
        // Recreate a BinaryArray so that we can force it to have geoarrow.wkb extension type
        Arc::new(self.into_arrow())
    }

    fn to_array_ref(&self) -> arrow_array::ArrayRef {
        self.clone().into_array_ref()
    }

    fn metadata(&self) -> Arc<ArrayMetadata> {
        self.metadata.clone()
    }

    /// Returns the number of geometries in this array
    #[inline]
    fn len(&self) -> usize {
        self.array.len()
    }

    /// Returns the optional validity.
    fn nulls(&self) -> Option<&NullBuffer> {
        self.array.nulls()
    }
}

impl<O: OffsetSizeTrait> SerializedArray for WKBArray<O> {
    fn data_type(&self) -> SerializedType {
        self.data_type
    }

    fn with_metadata(&self, metadata: Arc<ArrayMetadata>) -> Arc<dyn SerializedArray> {
        Arc::new(self.with_metadata(metadata))
    }

    fn as_ref(&self) -> &dyn SerializedArray {
        self
    }
}

impl<'a, O: OffsetSizeTrait> ArrayAccessor<'a> for WKBArray<O> {
    type Item = WKB<'a, O>;
    type ItemGeo = geo::Geometry;

    unsafe fn value_unchecked(&'a self, index: usize) -> Self::Item {
        WKB::new(&self.array, index)
    }
}

impl<O: OffsetSizeTrait> IntoArrow for WKBArray<O> {
    type ArrowArray = GenericBinaryArray<O>;

    fn into_arrow(self) -> Self::ArrowArray {
        GenericBinaryArray::new(
            self.array.offsets().clone(),
            self.array.values().clone(),
            self.array.nulls().cloned(),
        )
    }
}

impl<O: OffsetSizeTrait> From<GenericBinaryArray<O>> for WKBArray<O> {
    fn from(value: GenericBinaryArray<O>) -> Self {
        Self::new(value, Default::default())
    }
}

impl TryFrom<&dyn Array> for WKBArray<i32> {
    type Error = GeoArrowError;
    fn try_from(value: &dyn Array) -> Result<Self> {
        match value.data_type() {
            DataType::Binary => {
                let downcasted = value.as_any().downcast_ref::<BinaryArray>().unwrap();
                Ok(downcasted.clone().into())
            }
            DataType::LargeBinary => {
                let downcasted = value.as_any().downcast_ref::<LargeBinaryArray>().unwrap();
                let geom_array: WKBArray<i64> = downcasted.clone().into();
                geom_array.try_into()
            }
            _ => Err(GeoArrowError::General(format!(
                "Unexpected type: {:?}",
                value.data_type()
            ))),
        }
    }
}

impl TryFrom<&dyn Array> for WKBArray<i64> {
    type Error = GeoArrowError;
    fn try_from(value: &dyn Array) -> Result<Self> {
        match value.data_type() {
            DataType::Binary => {
                let downcasted = value.as_binary::<i32>();
                let geom_array: WKBArray<i32> = downcasted.clone().into();
                Ok(geom_array.into())
            }
            DataType::LargeBinary => {
                let downcasted = value.as_binary::<i64>();
                Ok(downcasted.clone().into())
            }
            _ => Err(GeoArrowError::General(format!(
                "Unexpected type: {:?}",
                value.data_type()
            ))),
        }
    }
}

impl TryFrom<(&dyn Array, &Field)> for WKBArray<i32> {
    type Error = GeoArrowError;

    fn try_from((arr, field): (&dyn Array, &Field)) -> Result<Self> {
        let mut arr: Self = arr.try_into()?;
        arr.metadata = Arc::new(ArrayMetadata::try_from(field)?);
        Ok(arr)
    }
}

impl TryFrom<(&dyn Array, &Field)> for WKBArray<i64> {
    type Error = GeoArrowError;

    fn try_from((arr, field): (&dyn Array, &Field)) -> Result<Self> {
        let mut arr: Self = arr.try_into()?;
        arr.metadata = Arc::new(ArrayMetadata::try_from(field)?);
        Ok(arr)
    }
}

impl From<WKBArray<i32>> for WKBArray<i64> {
    fn from(value: WKBArray<i32>) -> Self {
        let binary_array = value.array;
        let (offsets, values, nulls) = binary_array.into_parts();
        Self::new(
            LargeBinaryArray::new(offsets_buffer_i32_to_i64(&offsets), values, nulls),
            value.metadata,
        )
    }
}

impl TryFrom<WKBArray<i64>> for WKBArray<i32> {
    type Error = GeoArrowError;

    fn try_from(value: WKBArray<i64>) -> Result<Self> {
        let binary_array = value.array;
        let (offsets, values, nulls) = binary_array.into_parts();
        Ok(Self::new(
            BinaryArray::new(offsets_buffer_i64_to_i32(&offsets)?, values, nulls),
            value.metadata,
        ))
    }
}

// impl TryFrom<&BinaryArray<i64>> for WKBArray {
//     type Error = GeoArrowError;

//     fn try_from(value: &BinaryArray<i64>) -> Result<Self, Self::Error> {
//         Ok(Self::new(value.clone()))
//     }
// }

// impl TryFrom<&dyn Array> for WKBArray {
//     type Error = GeoArrowError;

//     fn try_from(value: &dyn Array) -> Result<Self, Self::Error> {
//         match value.data_type() {
//             DataType::Binary => {
//                 let downcasted = value.as_any().downcast_ref::<BinaryArray<i32>>().unwrap();
//                 downcasted.try_into()
//             }
//             DataType::LargeBinary => {
//                 let downcasted = value.as_any().downcast_ref::<BinaryArray<i64>>().unwrap();
//                 downcasted.try_into()
//             }
//             _ => Err(GeoArrowError::General(format!(
//                 "Unexpected type: {:?}",
//                 value.data_type()
//             ))),
//         }
//     }
// }

impl<O: OffsetSizeTrait, G: GeometryTrait<T = f64>> TryFrom<&[G]> for WKBArray<O> {
    type Error = GeoArrowError;

    fn try_from(geoms: &[G]) -> Result<Self> {
        let mut_arr: WKBBuilder<O> = geoms.try_into()?;
        Ok(mut_arr.into())
    }
}

impl<O: OffsetSizeTrait, G: GeometryTrait<T = f64>> TryFrom<Vec<Option<G>>> for WKBArray<O> {
    type Error = GeoArrowError;

    fn try_from(geoms: Vec<Option<G>>) -> Result<Self> {
        let mut_arr: WKBBuilder<O> = geoms.try_into()?;
        Ok(mut_arr.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow_array::BinaryArray;

    #[test]
    fn issue_243() {
        let binary_arr = BinaryArray::from_opt_vec(vec![None]);
        let wkb_arr = WKBArray::from(binary_arr);

        // We just need to ensure that the iterator runs
        wkb_arr.iter_geo().for_each(|_x| ());
    }
}
