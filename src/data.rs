mod directed_edge;
mod line_segment;
pub(crate) mod point;
pub mod polygon;
mod triangle;
mod vector;
pub mod vertex;

pub use directed_edge::*;
pub use line_segment::*;
pub use triangle::*;

// pub use crate::polygon::EdgeIter as testing;

#[doc(inline)]
pub use crate::data::polygon::{Polygon, PolygonConvex};
pub use crate::transformation::Transform;
pub use point::Point;
pub use vector::{Vector, VectorView};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointLocation {
  Inside,
  OnBoundary,
  Outside,
}
