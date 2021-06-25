use num_traits::*;

use super::EndPoint;
use super::ILineSegment;
use super::LineSegment;
use super::LineSegmentView;
use super::Point;
use crate::Intersects;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
// Directed edge from A to B, including A and excluding B.
pub struct DirectedEdge<T, const N: usize> {
  pub src: Point<T, N>,
  pub dst: Point<T, N>,
}

impl<T: Ord, const N: usize> From<DirectedEdge<T, N>> for LineSegment<T, N> {
  fn from(edge: DirectedEdge<T, N>) -> LineSegment<T, N> {
    LineSegment::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

impl<'a, T: Ord, const N: usize> From<&'a DirectedEdge<T, N>> for LineSegmentView<'a, T, N> {
  fn from(edge: &'a DirectedEdge<T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(&edge.src),
      EndPoint::Exclusive(&edge.dst),
    )
  }
}

impl<'a, T> Intersects for &'a DirectedEdge<T, 2>
where
  T: Clone + Num + Ord + std::fmt::Debug + crate::Extended,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a DirectedEdge<T, 2>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}
