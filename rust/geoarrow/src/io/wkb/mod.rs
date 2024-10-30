//! An optimized implementation of reading and writing ISO-flavored WKB-encoded geometries.

mod api;

pub use api::{from_wkb, to_wkb, FromWKB, ToWKB};
pub use common::WKBType;
