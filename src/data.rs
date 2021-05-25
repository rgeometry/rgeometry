mod line_segment;
mod point;
pub mod polygon;
mod triangle;
mod vector;

pub use line_segment::*;
pub use triangle::*;

// pub use crate::polygon::EdgeIter as testing;

#[doc(inline)]
pub use crate::data::polygon::{ConvexPolygon, Polygon};
pub use crate::transformation::Transform;
pub use point::Point;
pub use vector::{Vector, VectorView};

#[derive(Debug, Clone, Copy)]
pub enum PointLocation {
  Inside,
  OnBoundary,
  Outside,
}
