use super::EndPoint;
use super::ILineSegment;
use super::LineSegment;
use super::LineSegmentView;
use super::Point;
use crate::{Intersects, PolygonScalar, TotalOrd};

///////////////////////////////////////////////////////////////////////////////
// DirectedEdge

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
// Directed edge from A to B, including A and excluding B.
pub struct DirectedEdge<T: TotalOrd, const N: usize> {
  pub src: Point<T, N>,
  pub dst: Point<T, N>,
}

// DirectedEdge -> LineSegment
impl<T: TotalOrd, const N: usize> From<DirectedEdge<T, N>> for LineSegment<T, N> {
  fn from(edge: DirectedEdge<T, N>) -> LineSegment<T, N> {
    LineSegment::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

// DirectedEdge -> LineSegmentView
impl<'a, T: TotalOrd, const N: usize> From<&'a DirectedEdge<T, N>> for LineSegmentView<'a, T, N> {
  fn from(edge: &'a DirectedEdge<T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(&edge.src),
      EndPoint::Exclusive(&edge.dst),
    )
  }
}

impl<'a, T> Intersects for &'a DirectedEdge<T, 2>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a DirectedEdge<T, 2>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}

///////////////////////////////////////////////////////////////////////////////
// DirectedEdgeView

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
// Directed edge from A to B, including A and excluding B.
pub struct DirectedEdgeView<'a, T: TotalOrd, const N: usize = 2> {
  pub src: &'a Point<T, N>,
  pub dst: &'a Point<T, N>,
}

impl<T: TotalOrd, const N: usize> Copy for DirectedEdgeView<'_, T, N> {}
impl<T: TotalOrd, const N: usize> Clone for DirectedEdgeView<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: TotalOrd> DirectedEdgeView<'_, T, 2> {
  pub fn contains(self, pt: &Point<T, 2>) -> bool
  where
    T: PolygonScalar,
  {
    LineSegmentView::from(self).contains(pt)
  }
}

impl<'a, T: TotalOrd, const N: usize> From<DirectedEdgeView<'a, T, N>>
  for LineSegmentView<'a, T, N>
{
  fn from(edge: DirectedEdgeView<'a, T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

impl<'a, T: TotalOrd, const N: usize> From<&DirectedEdgeView<'a, T, N>>
  for LineSegmentView<'a, T, N>
{
  fn from(edge: &DirectedEdgeView<'a, T, N>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(EndPoint::Inclusive(edge.src), EndPoint::Exclusive(edge.dst))
  }
}

impl<'a, T> Intersects for DirectedEdgeView<'a, T, 2>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: DirectedEdgeView<'a, T, 2>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}
