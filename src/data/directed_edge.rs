use super::EndPoint;
use super::ILineSegment;
use super::LineSegment;
use super::LineSegmentView;
use super::Point;
use crate::{Intersects, PolygonScalar, TotalOrd};

///////////////////////////////////////////////////////////////////////////////
// DirectedEdge_

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
// Directed edge from A to B, including A and excluding B.
pub struct DirectedEdge_<T: TotalOrd, const N: usize> {
  pub src: Point<T, N>,
  pub dst: Point<T, N>,
}

// DirectedEdge_ -> LineSegment
impl<T: TotalOrd, const N: usize> From<DirectedEdge_<T, N>> for LineSegment<T, N> {
  fn from(edge: DirectedEdge_<T, N>) -> LineSegment<T, N> {
    LineSegment::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

// DirectedEdge_ -> LineSegmentView
impl<'a, T: TotalOrd, const N: usize> From<&'a DirectedEdge_<T, N>> for LineSegmentView<'a, T, N> {
  fn from(edge: &'a DirectedEdge_<T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(&edge.src),
      EndPoint::Exclusive(&edge.dst),
    )
  }
}

impl<'a, T> Intersects for &'a DirectedEdge_<T, 2>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a DirectedEdge_<T, 2>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}

///////////////////////////////////////////////////////////////////////////////
// DirectedEdge

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
// Directed edge from A to B, including A and excluding B.
pub struct DirectedEdge<'a, T: TotalOrd, const N: usize = 2> {
  pub src: &'a Point<T, N>,
  pub dst: &'a Point<T, N>,
}

impl<'a, T: TotalOrd, const N: usize> Copy for DirectedEdge<'a, T, N> {}
impl<'a, T: TotalOrd, const N: usize> Clone for DirectedEdge<'a, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, T: TotalOrd> DirectedEdge<'a, T, 2> {
  pub fn contains(self, pt: &Point<T, 2>) -> bool
  where
    T: PolygonScalar,
  {
    LineSegmentView::from(self).contains(pt)
  }
}

impl<'a, T: TotalOrd, const N: usize> From<DirectedEdge<'a, T, N>> for LineSegmentView<'a, T, N> {
  fn from(edge: DirectedEdge<'a, T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

impl<'a, T: TotalOrd, const N: usize> From<&DirectedEdge<'a, T, N>> for LineSegmentView<'a, T, N> {
  fn from(edge: &DirectedEdge<'a, T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

impl<'a, T> Intersects for DirectedEdge<'a, T, 2>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: DirectedEdge<'a, T, 2>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}
