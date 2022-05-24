use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::ops::Range;
use std::ops::RangeInclusive;

use super::Point;

use crate::data::point::PointSoS;
use crate::Intersects;
use crate::{Orientation, PolygonScalar};
use Orientation::*;

///////////////////////////////////////////////////////////////////////////////
// EndPoint

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EndPoint<T> {
  Exclusive(T),
  Inclusive(T),
}

use EndPoint::*;

impl<T> EndPoint<T> {
  pub fn inner(&self) -> &T {
    match self {
      Exclusive(t) => t,
      Inclusive(t) => t,
    }
  }

  pub fn take(self) -> T {
    match self {
      Exclusive(t) => t,
      Inclusive(t) => t,
    }
  }

  pub fn as_ref(&self) -> EndPoint<&'_ T> {
    match self {
      Exclusive(t) => Exclusive(t),
      Inclusive(t) => Inclusive(t),
    }
  }

  pub fn is_exclusive(&self) -> bool {
    match self {
      Exclusive(_) => true,
      Inclusive(_) => false,
    }
  }

  pub fn is_inclusive(&self) -> bool {
    !self.is_exclusive()
  }

  #[must_use]
  pub fn leftmost(self, other: EndPoint<T>) -> EndPoint<T>
  where
    T: Ord,
  {
    match self.inner().cmp(other.inner()) {
      Ordering::Equal => {
        if self.is_exclusive() {
          self
        } else {
          other
        }
      }
      Ordering::Less => self,
      Ordering::Greater => other,
    }
  }

  #[must_use]
  pub fn rightmost(self, other: EndPoint<T>) -> EndPoint<T>
  where
    T: Ord,
  {
    match self.inner().cmp(other.inner()) {
      Ordering::Equal => {
        if self.is_exclusive() || other.is_exclusive() {
          Exclusive(self.take())
        } else {
          other
        }
      }
      Ordering::Less => other,
      Ordering::Greater => self,
    }
  }
}

fn inner_between<T: PartialOrd>(inner: &T, a: EndPoint<&T>, b: EndPoint<&T>) -> bool {
  let left;
  let right;
  if a.inner() < b.inner() {
    left = a;
    right = b;
  } else {
    left = b;
    right = a;
  }
  let lhs = match left {
    Exclusive(l) => l < inner,
    Inclusive(l) => l <= inner,
  };
  let rhs = match right {
    Exclusive(r) => inner < r,
    Inclusive(r) => inner <= r,
  };
  lhs && rhs
}

///////////////////////////////////////////////////////////////////////////////
// LineSegment

#[derive(Debug, Clone, Copy)]
pub struct LineSegment<T, const N: usize = 2> {
  pub min: EndPoint<Point<T, N>>,
  pub max: EndPoint<Point<T, N>>,
}

impl<T, const N: usize> LineSegment<T, N> {
  pub fn new(a: EndPoint<Point<T, N>>, b: EndPoint<Point<T, N>>) -> LineSegment<T, N>
  where
    T: Ord,
  {
    if a.inner() < b.inner() {
      LineSegment { min: a, max: b }
    } else {
      LineSegment { min: b, max: a }
    }
  }

  pub fn as_ref(&self) -> LineSegmentView<'_, T, N> {
    LineSegmentView {
      min: self.min.as_ref(),
      max: self.max.as_ref(),
    }
  }
}
impl<T> LineSegment<T> {
  pub fn contains(&self, pt: &Point<T>) -> bool
  where
    T: PolygonScalar,
  {
    self.as_ref().contains(pt)
  }
}

impl<T: Ord, const N: usize> From<Range<Point<T, N>>> for LineSegment<T, N> {
  fn from(range: Range<Point<T, N>>) -> LineSegment<T, N> {
    LineSegment::new(
      EndPoint::Inclusive(range.start),
      EndPoint::Exclusive(range.end),
    )
  }
}

impl<T: Ord> From<Range<(T, T)>> for LineSegment<T> {
  fn from(range: Range<(T, T)>) -> LineSegment<T> {
    LineSegment::new(
      EndPoint::Inclusive(range.start.into()),
      EndPoint::Exclusive(range.end.into()),
    )
  }
}

impl<T: Ord, const N: usize> From<RangeInclusive<Point<T, N>>> for LineSegment<T, N> {
  fn from(range: RangeInclusive<Point<T, N>>) -> LineSegment<T, N> {
    let (start, end) = range.into_inner();
    LineSegment::new(EndPoint::Inclusive(start), EndPoint::Inclusive(end))
  }
}

///////////////////////////////////////////////////////////////////////////////
// LineSegmentView

#[derive(Debug, PartialEq, Eq)]
pub struct LineSegmentView<'a, T, const N: usize = 2> {
  pub min: EndPoint<&'a Point<T, N>>,
  pub max: EndPoint<&'a Point<T, N>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LineSegmentSoS<'a, T, const N: usize> {
  pub min: EndPoint<PointSoS<'a, T, N>>,
  pub max: EndPoint<PointSoS<'a, T, N>>,
}

impl<'a, T, const N: usize> Clone for LineSegmentView<'a, T, N> {
  fn clone(&self) -> Self {
    LineSegmentView {
      min: self.min,
      max: self.max,
    }
  }
}
impl<'a, T, const N: usize> Copy for LineSegmentView<'a, T, N> {}

impl<'a, T, const N: usize> LineSegmentView<'a, T, N> {
  pub fn new(
    a: EndPoint<&'a Point<T, N>>,
    b: EndPoint<&'a Point<T, N>>,
  ) -> LineSegmentView<'a, T, N>
  where
    T: Ord,
  {
    if a.inner() < b.inner() {
      LineSegmentView { min: a, max: b }
    } else {
      LineSegmentView { min: b, max: a }
    }
  }
}

impl<'a, T> LineSegmentView<'a, T> {
  pub fn contains(&self, pt: &Point<T>) -> bool
  where
    T: PolygonScalar,
  {
    Point::orient(self.min.inner(), self.max.inner(), pt).is_colinear()
      && inner_between(pt, self.min, self.max)
  }
}

impl<'a, T: Ord, const N: usize> From<&'a Range<Point<T, N>>> for LineSegmentView<'a, T, N> {
  fn from(range: &'a Range<Point<T, N>>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(&range.start),
      EndPoint::Exclusive(&range.end),
    )
  }
}

impl<'a, T: Ord, const N: usize> From<Range<&'a Point<T, N>>> for LineSegmentView<'a, T, N> {
  fn from(range: Range<&'a Point<T, N>>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(range.start),
      EndPoint::Exclusive(range.end),
    )
  }
}

impl<'a, T: Ord, const N: usize> From<&'a RangeInclusive<Point<T, N>>>
  for LineSegmentView<'a, T, N>
{
  fn from(range: &'a RangeInclusive<Point<T, N>>) -> LineSegmentView<'a, T, N> {
    LineSegmentView::new(
      EndPoint::Inclusive(range.start()),
      EndPoint::Inclusive(range.end()),
    )
  }
}

///////////////////////////////////////////////////////////////////////////////
// ILineSegment

#[derive(Debug, Eq, PartialEq)]
pub enum ILineSegment<'a, T> {
  Crossing,                        // Lines touch but are not parallel.
  Overlap(LineSegmentView<'a, T>), // Lines touch and are parallel.
}

///////////////////////////////////////////////////////////////////////////////
// Intersects

impl<'a, T> Intersects for LineSegmentView<'a, T>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: LineSegmentView<'a, T>) -> Option<Self::Result> {
    let a1 = self.min.inner();
    let a2 = self.max.inner();
    let b1 = other.min.inner();
    let b2 = other.max.inner();
    let l1_to_b1 = Point::orient(a1, a2, b1);
    let l1_to_b2 = Point::orient(a1, a2, b2);
    let l2_to_a1 = Point::orient(b1, b2, a1);
    let l2_to_a2 = Point::orient(b1, b2, a2);
    // dbg!(
    //   a1,
    //   a2,
    //   b1,
    //   b2,
    //   l1_to_b1,
    //   l1_to_b2,
    //   l2_to_a1,
    //   l2_to_a2,
    //   inner_between(other.min.inner(), self.min.as_ref(), self.max.as_ref()),
    //   inner_between(other.max.inner(), self.min.as_ref(), self.max.as_ref()),
    //   inner_between(self.min.inner(), other.min.as_ref(), other.max.as_ref()),
    //   inner_between(self.max.inner(), other.min.as_ref(), other.max.as_ref()),
    // );
    if l1_to_b1 == CoLinear && l1_to_b2 == CoLinear {
      let c_min = self.min.rightmost(other.min);
      let c_max = self.max.leftmost(other.max);
      let order = c_min.inner().cmp(c_max.inner());
      if order == Ordering::Less
        || (order == Ordering::Equal && c_max.is_inclusive() && c_min.is_inclusive())
      {
        Some(ILineSegment::Overlap(LineSegmentView {
          min: c_min,
          max: c_max,
        }))
      } else {
        None
      }
    } else if l1_to_b1 == CoLinear {
      // dbg!(other.0 >= self.0, other.0 <= self.1, other.0.is_closed());
      if other.min.is_inclusive()
        && inner_between(other.min.inner(), self.min.as_ref(), self.max.as_ref())
      {
        Some(ILineSegment::Crossing)
      } else {
        None
      }
    } else if l1_to_b2 == CoLinear {
      if other.max.is_inclusive()
        && inner_between(other.max.inner(), self.min.as_ref(), self.max.as_ref())
      {
        Some(ILineSegment::Crossing)
      } else {
        None
      }
    } else if l2_to_a1 == CoLinear {
      if self.min.is_inclusive()
        && inner_between(self.min.inner(), other.min.as_ref(), other.max.as_ref())
      {
        Some(ILineSegment::Crossing)
      } else {
        None
      }
    } else if l2_to_a2 == CoLinear {
      if self.max.is_inclusive()
        && inner_between(self.max.inner(), other.min.as_ref(), other.max.as_ref())
      {
        Some(ILineSegment::Crossing)
      } else {
        None
      }
    } else if l1_to_b1 == l1_to_b2.reverse() && l2_to_a1 == l2_to_a2.reverse() {
      Some(ILineSegment::Crossing)
    } else {
      None
    }
  }
}

impl<'a, T> Intersects for &'a LineSegment<T>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a LineSegment<T>) -> Option<Self::Result> {
    self.as_ref().intersect(other.as_ref())
  }
}

impl<'a, T> Intersects for &'a Range<Point<T>>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a Range<Point<T>>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}

impl<'a, T> Intersects for &'a RangeInclusive<Point<T>>
where
  T: PolygonScalar,
{
  type Result = ILineSegment<'a, T>;
  fn intersect(self, other: &'a RangeInclusive<Point<T>>) -> Option<Self::Result> {
    LineSegmentView::from(self).intersect(LineSegmentView::from(other))
  }
}

///////////////////////////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::*;
  use crate::Intersects;
  use ILineSegment::*;

  use test_strategy::proptest;

  #[proptest]
  fn flip_intersects_prop(pts: [i8; 8]) {
    let [a, b, c, d, e, f, g, h] = pts;
    let l1 = LineSegment::from((a, b)..(c, d));
    let l2 = LineSegment::from((e, f)..(g, h));
    assert_eq!(l1.intersect(&l2), l2.intersect(&l1));
  }

  //             P6
  //
  // P7      P5
  //
  // P4  P2
  //
  // P1  P3
  //
  static P1: Point<i32> = Point::new([0, 0]);
  static P2: Point<i32> = Point::new([1, 1]);
  static P3: Point<i32> = Point::new([1, 0]);
  static P4: Point<i32> = Point::new([0, 1]);
  static P5: Point<i32> = Point::new([2, 2]);
  static P6: Point<i32> = Point::new([3, 3]);
  static P7: Point<i32> = Point::new([0, 2]);

  #[test]
  fn line_crossing() {
    assert_eq!((P1..P2).intersect(&(P3..P4)), Some(Crossing))
  }

  #[test]
  fn line_not_crossing() {
    assert_eq!((P1..P3).intersect(&(P2..P4)), None)
  }

  #[test]
  fn endpoints_no_overlap() {
    assert_eq!((P1..P2).intersect(&(P2..P3)), None)
  }

  #[test]
  fn endpoints_overlap_1() {
    assert_eq!((P2..P1).intersect(&(P2..P3)), Some(Crossing))
  }

  #[test]
  fn endpoints_overlap_2() {
    assert_eq!((P1..P2).intersect(&(P1..P3)), Some(Crossing))
  }

  #[test]
  fn endpoints_overlap_3() {
    assert_eq!(
      (P1..=P2).intersect(&(P2..=P5)),
      Some(Overlap(LineSegmentView::from(&(P2..=P2))))
    )
  }

  #[test]
  fn edges_overlap_1() {
    assert_eq!(
      (P1..P5).intersect(&(P2..P6)),
      Some(Overlap(LineSegmentView::from(&(P2..P5))))
    )
  }

  #[test]
  fn edges_overlap_2() {
    assert_eq!(
      (P1..P6).intersect(&(P2..P6)),
      Some(Overlap(LineSegmentView::from(&(P2..P6))))
    )
  }

  #[test]
  fn edges_overlap_3() {
    assert_eq!(
      (P6..P1).intersect(&(P6..P2)),
      Some(Overlap(LineSegmentView::from(&(P6..P2))))
    )
  }

  #[test]
  fn edges_overlap_4() {
    let l1 = LineSegment::new(Inclusive(P1), Exclusive(P5));
    let l2 = LineSegment::new(Exclusive(P2), Inclusive(P6));
    let l3 = LineSegment::new(Exclusive(P2), Exclusive(P5));
    assert_eq!(l1.intersect(&l2), Some(Overlap(l3.as_ref())))
  }

  #[test]
  fn edges_overlap_5() {
    let l1 = LineSegment::new(Inclusive(P1), Inclusive(P5));
    let l2 = LineSegment::new(Exclusive(P2), Inclusive(P6));
    let l3 = LineSegment::new(Exclusive(P2), Inclusive(P5));
    assert_eq!(l1.intersect(&l2), Some(Overlap(l3.as_ref())))
  }

  #[test]
  fn edge_touch() {
    assert_eq!((P1..P7).intersect(&(P4..P2)), Some(Crossing))
  }

  #[test]
  fn edge_no_touch() {
    assert_eq!((P1..P7).intersect(&(P2..P4)), None)
  }

  #[test]
  fn subset_1() {
    let l1 = LineSegment::new(Inclusive(P1), Inclusive(P2));
    let l2 = LineSegment::new(Exclusive(P1), Exclusive(P2));
    assert_eq!(l1.intersect(&l2), Some(Overlap(l2.as_ref())))
  }

  #[test]
  fn subset_2() {
    let l1 = LineSegment::new(Inclusive(P1), Exclusive(P2));
    let l2 = LineSegment::new(Exclusive(P1), Inclusive(P2));
    let l3 = LineSegment::new(Exclusive(P1), Exclusive(P2));
    assert_eq!(l1.intersect(&l2), Some(Overlap(l3.as_ref())))
  }

  #[test]
  fn unit_1() {
    let a1 = Point::new([1, 0]);
    let a2 = Point::new([1, 1]);
    let b1 = Point::new([0, 1]);
    let b2 = Point::new([2, 3]);
    let l1 = LineSegment::from(a1..a2);
    let l2 = LineSegment::from(b1..b2);
    assert_eq!(l1.intersect(&l2), None)
  }

  #[test]
  fn unit_2() {
    let a1 = Point::new([1, 0]);
    let a2 = Point::new([1, 1]);
    let b1 = Point::new([0, 0]);
    let b2 = Point::new([2, 3]);
    let l1 = LineSegment::from(b2..b1);
    let l2 = LineSegment::from(a1..a2);
    assert_eq!(l1.intersect(&l2), None)
  }

  #[test]
  fn unit_3() {
    let l1 = LineSegment::from((0, 0)..(1, 0));
    let l2 = LineSegment::from((1, 0)..(2, 0));
    assert_eq!(l1.intersect(&l2), None)
  }

  #[test]
  fn unit_4() {
    let l1 = &LineSegment::from((0, 0)..(1, 0));
    let l2 = &LineSegment::from((1, 0)..(2, 0));
    assert_eq!(l2.intersect(l1), None);
    assert_eq!(l1.intersect(l2), None);
  }

  #[test]
  fn unit_5() {
    let l1 = LineSegment::new(Inclusive((0, 0).into()), Inclusive((1, 0).into()));
    let l2 = LineSegment::new(Inclusive((1, 0).into()), Inclusive((2, 0).into()));
    let l3 = LineSegment::new(Inclusive((1, 0).into()), Inclusive((1, 0).into()));
    assert_eq!(l2.intersect(&l1), Some(Overlap(l3.as_ref())))
  }

  #[test]
  fn unit_6() {
    let l1 = LineSegment::from((4, 0)..(3, 0));
    let l2 = LineSegment::from((3, 0)..(1, 0));
    assert_eq!(l1.intersect(&l2), None)
  }

  #[test]
  fn unit_7() {
    let l1 = LineSegment::from((0, 0)..(0, 1));
    let l2 = LineSegment::from((1, 2)..(2, 1));
    assert_eq!(l1.intersect(&l2), None)
  }

  #[test]
  fn unit_8() {
    let l1 = LineSegment::from((-106, 0)..(54, -128));
    let l2 = LineSegment::from((-71, -28)..(31, -8));
    assert_eq!(l1.intersect(&l2), l2.intersect(&l1));
  }
}
