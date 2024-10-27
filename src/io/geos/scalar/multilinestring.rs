use crate::error::{GeoArrowError, Result};
use crate::io::geos::scalar::GEOSConstLineString;
use crate::scalar::MultiLineString;
use geo_traits::MultiLineStringTrait;
use geos::{Geom, GeometryTypes};

impl<'a, const D: usize> TryFrom<&'a MultiLineString<'_, D>> for geos::Geometry {
    type Error = geos::Error;

    fn try_from(
        value: &'a MultiLineString<'_, D>,
    ) -> std::result::Result<geos::Geometry, geos::Error> {
        geos::Geometry::create_multiline_string(
            value
                .line_strings()
                .map(|line| (&line).try_into())
                .collect::<std::result::Result<Vec<_>, geos::Error>>()?,
        )
    }
}
/// A GEOS geometry known to be a MultiLineString
#[derive(Clone)]
pub struct GEOSMultiLineString(pub(crate) geos::Geometry);

impl GEOSMultiLineString {
    pub fn new_unchecked(geom: geos::Geometry) -> Self {
        Self(geom)
    }

    #[allow(dead_code)]
    pub fn try_new(geom: geos::Geometry) -> Result<Self> {
        if matches!(geom.geometry_type(), GeometryTypes::MultiLineString) {
            Ok(Self(geom))
        } else {
            Err(GeoArrowError::General(
                "Geometry type must be multi line string".to_string(),
            ))
        }
    }

    pub fn num_lines(&self) -> usize {
        self.0.get_num_geometries().unwrap()
    }

    #[allow(dead_code)]
    pub fn line(&self, i: usize) -> Option<GEOSConstLineString<'_>> {
        if i > (self.num_lines()) {
            return None;
        }

        Some(GEOSConstLineString::new_unchecked(
            self.0.get_geometry_n(i).unwrap(),
        ))
    }
}

impl MultiLineStringTrait for GEOSMultiLineString {
    type T = f64;
    type LineStringType<'a> = GEOSConstLineString<'a> where Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        match self.0.get_coordinate_dimension().unwrap() {
            geos::Dimensions::TwoD => geo_traits::Dimensions::Xy,
            geos::Dimensions::ThreeD => geo_traits::Dimensions::Xyz,
            geos::Dimensions::Other(other) => panic!("Other dimensions not supported {other}"),
        }
    }

    fn num_line_strings(&self) -> usize {
        self.0.get_num_geometries().unwrap()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        GEOSConstLineString::new_unchecked(self.0.get_geometry_n(i).unwrap())
    }
}
