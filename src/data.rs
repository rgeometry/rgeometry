mod directed_edge;
mod intersection_set;
mod line_segment;
pub(crate) mod point;
pub mod polygon;
mod triangle;
mod vector;

pub use directed_edge::*;
pub use intersection_set::*;
pub use line_segment::*;
pub use triangle::*;

// pub use crate::polygon::EdgeIter as testing;

#[doc(inline)]
pub use crate::data::polygon::{
  Cursor, DirectedIndexEdge, IndexEdge, PointId, Polygon, PolygonConvex, Position, PositionId,
  RingId,
};
pub use crate::transformation::Transform;
pub use point::Point;
pub use vector::{Vector, VectorView};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointLocation {
  Inside,
  OnBoundary,
  Outside,
}
