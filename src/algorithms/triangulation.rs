use crate::data::{PointId, Polygon};
use crate::PolygonScalar;

pub mod earclip;

// FIXME: impl for PolygonConvex, Triangle, etc.
pub trait Triangulate {
  fn triangulate(&self) -> Box<dyn Iterator<Item = (PointId, PointId, PointId)> + '_>;
}

// FIXME: Use hashed triangulation by default.
impl<T: PolygonScalar> Triangulate for Polygon<T> {
  fn triangulate(&self) -> Box<dyn Iterator<Item = (PointId, PointId, PointId)> + '_> {
    Box::new(earclip::earclip(self))
  }
}
