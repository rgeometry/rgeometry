use crate::data::{PointId, Polygon};
use crate::PolygonScalar;

pub mod earclip;

// FIXME: impl for PolygonConvex, Triangle, etc.
pub trait Triangulate {
  type Iter: Iterator<Item = (PointId, PointId, PointId)>;
  fn triangulate(self) -> Self::Iter;
}

// FIXME: Use hashed triangulation by default.
impl<'a, T: PolygonScalar> Triangulate for &'a Polygon<T> {
  type Iter = Box<dyn Iterator<Item = (PointId, PointId, PointId)> + 'a>;
  fn triangulate(self) -> Self::Iter {
    Box::new(earclip::earclip(self))
  }
}
