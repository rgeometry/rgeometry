use super::DirectedEdgeView;
use super::LineSegmentView;
use super::Point;
use super::Vector;
use crate::Intersects;
use crate::TotalOrd;
use crate::data::LineSegment;
use crate::{Orientation, PolygonScalar};

///////////////////////////////////////////////////////////////////////////////
// LineView

#[derive(Debug)]
pub struct LineView<'a, T, const N: usize = 2> {
  pub origin: &'a Point<T, N>,
  pub direction: DirectionView<'a, T, N>,
}

impl<T, const N: usize> Copy for LineView<'_, T, N> {}
impl<T, const N: usize> Clone for LineView<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, T: TotalOrd, const N: usize> From<&'a LineSegment<T, N>> for LineView<'a, T, N> {
  fn from(segment: &'a LineSegment<T, N>) -> Self {
    LineView::new_through(segment.min.inner(), segment.max.inner())
  }
}

impl<'a, T, const N: usize> LineView<'a, T, N> {
  pub fn new(origin: &'a Point<T, N>, direction: DirectionView<'a, T, N>) -> LineView<'a, T, N> {
    LineView { origin, direction }
  }

  pub fn new_directed(origin: &'a Point<T, N>, vector: &'a Vector<T, N>) -> LineView<'a, T, N> {
    LineView::new(origin, DirectionView::Vector(vector))
  }

  pub fn new_through(origin: &'a Point<T, N>, through: &'a Point<T, N>) -> LineView<'a, T, N> {
    LineView::new(origin, DirectionView::Through(through))
  }
}

impl<T: PolygonScalar> LineView<'_, T> {
  pub fn intersection_point(&self, other: &Self) -> Option<Point<T>> {
    let [x1, y1] = self.origin.array.clone();
    let [x2, y2] = match &self.direction {
      DirectionView::Through(pt) => pt.array.clone(),
      _ => unimplemented!(),
    };
    let [x3, y3] = other.origin.array.clone();
    let [x4, y4] = match &other.direction {
      DirectionView::Through(pt) => pt.array.clone(),
      _ => unimplemented!(),
    };
    T::approx_intersection_point([x1, y1], [x2, y2], [x3, y3], [x4, y4])
      .map(|[x, y]| Point::new([x, y]))
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line

#[derive(Debug, Clone)]
pub struct Line<T, const N: usize> {
  pub origin: Point<T, N>,
  pub direction: Direction<T, N>,
}

impl<'a, T, const N: usize> From<&'a Line<T, N>> for LineView<'a, T, N> {
  fn from(line: &'a Line<T, N>) -> LineView<'a, T, N> {
    LineView {
      origin: &line.origin,
      direction: (&line.direction).into(),
    }
  }
}

// impl<'a, T: PolygonScalar> LineView<'a, T, 2> {
//   pub fn intersection_point(&self, other: &Self) -> Option<Point<T, 2>> {

//   }
// }

///////////////////////////////////////////////////////////////////////////////
// DirectionView

#[derive(Debug)]
pub enum DirectionView<'a, T, const N: usize> {
  Vector(&'a Vector<T, N>),
  Through(&'a Point<T, N>),
}

impl<T, const N: usize> Copy for DirectionView<'_, T, N> {}
impl<T, const N: usize> Clone for DirectionView<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

///////////////////////////////////////////////////////////////////////////////
// Direction

#[derive(Debug, Clone)]
pub enum Direction<T, const N: usize> {
  Vector(Vector<T, N>),
  Through(Point<T, N>),
}

impl<'a, T, const N: usize> From<&'a Direction<T, N>> for DirectionView<'a, T, N> {
  fn from(owned: &'a Direction<T, N>) -> DirectionView<'a, T, N> {
    match owned {
      Direction::Vector(v) => DirectionView::Vector(v),
      Direction::Through(p) => DirectionView::Through(p),
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// LineSoSView

#[derive(Debug, Clone)]
pub struct LineSoSView<'a, T, const N: usize = 2> {
  line: LineView<'a, T, N>,
}

impl<'a, T, const N: usize> From<LineSoSView<'a, T, N>> for LineView<'a, T, N> {
  fn from(sos: LineSoSView<'a, T, N>) -> LineView<'a, T, N> {
    sos.line
  }
}

impl<'a, T, const N: usize> From<LineView<'a, T, N>> for LineSoSView<'a, T, N> {
  fn from(line: LineView<'a, T, N>) -> LineSoSView<'a, T, N> {
    LineSoSView { line }
  }
}

///////////////////////////////////////////////////////////////////////////////
// LineSoS

#[derive(Debug, Clone)]
pub struct LineSoS<T, const N: usize = 2> {
  line: Line<T, N>,
}

impl<'a, T, const N: usize> From<&'a LineSoS<T, N>> for LineSoSView<'a, T, N> {
  fn from(line: &'a LineSoS<T, N>) -> LineSoSView<'a, T, N> {
    LineSoSView {
      line: LineView::from(&line.line),
    }
  }
}

impl<T, const N: usize> From<Line<T, N>> for LineSoS<T, N> {
  fn from(line: Line<T, N>) -> LineSoS<T, N> {
    LineSoS { line }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS intersection

pub enum ILineLineSegmentSoS {
  Crossing,
}

// Line / LineSegment intersection.
// If the line is colinear with an endpoint in the linesegment, the endpoint
// will be considered to be to the left of the line.
impl<T> Intersects<LineSegmentView<'_, T>> for &LineSoSView<'_, T>
where
  T: PolygonScalar,
{
  type Result = ILineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T>) -> Option<Self::Result> {
    let b1 = other.min.inner();
    let b2 = other.max.inner();
    let origin = &self.line.origin;
    let l1_to_b1;
    let l1_to_b2;
    match &self.line.direction {
      DirectionView::Vector(direction) => {
        l1_to_b1 =
          Point::orient_along_vector(origin, direction, b1).then(Orientation::CounterClockWise);
        l1_to_b2 =
          Point::orient_along_vector(origin, direction, b2).then(Orientation::CounterClockWise);
      }
      DirectionView::Through(through) => {
        l1_to_b1 = Point::orient(origin, through, b1).then(Orientation::CounterClockWise);
        l1_to_b2 = Point::orient(origin, through, b2).then(Orientation::CounterClockWise);
      }
    }
    // If b1 and b2 are on opposite sides of the line then there's an intersection.
    if l1_to_b1 == l1_to_b2.reverse() {
      Some(ILineLineSegmentSoS::Crossing)
    } else {
      None
    }
  }
}

impl<T> Intersects<DirectedEdgeView<'_, T>> for &LineSoSView<'_, T>
where
  T: PolygonScalar,
{
  type Result = ILineLineSegmentSoS;
  fn intersect(self, other: DirectedEdgeView<'_, T>) -> Option<Self::Result> {
    let line_segment: LineSegmentView<'_, T> = other.into();
    self.intersect(line_segment)
  }
}

///////////////////////////////////////////////////////////////////////////////
// Half-Line SoS

pub struct HalfLineSoS<'a, T, const N: usize = 2> {
  line: LineView<'a, T, N>,
}

impl<T, const N: usize> Copy for HalfLineSoS<'_, T, N> {}
impl<T, const N: usize> Clone for HalfLineSoS<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, T, const N: usize> From<LineSoSView<'a, T, N>> for HalfLineSoS<'a, T, N> {
  fn from(sos: LineSoSView<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    HalfLineSoS { line: sos.into() }
  }
}

impl<'a, T, const N: usize> From<LineView<'a, T, N>> for HalfLineSoS<'a, T, N> {
  fn from(line: LineView<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    HalfLineSoS { line }
  }
}

impl<'a, T, const N: usize> From<HalfLineSoS<'a, T, N>> for LineSoSView<'a, T, N> {
  fn from(ray: HalfLineSoS<'a, T, N>) -> LineSoSView<'a, T, N> {
    ray.line.into()
  }
}

impl<'a, T, const N: usize> From<HalfLineSoS<'a, T, N>> for LineView<'a, T, N> {
  fn from(ray: HalfLineSoS<'a, T, N>) -> LineView<'a, T, N> {
    ray.line
  }
}

impl<'a, T, const N: usize> HalfLineSoS<'a, T, N> {
  pub fn new(origin: &'a Point<T, N>, direction: DirectionView<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    LineView::new(origin, direction).into()
  }

  pub fn new_directed(origin: &'a Point<T, N>, vector: &'a Vector<T, N>) -> HalfLineSoS<'a, T, N> {
    LineView::new_directed(origin, vector).into()
  }

  pub fn new_through(origin: &'a Point<T, N>, through: &'a Point<T, N>) -> HalfLineSoS<'a, T, N> {
    LineView::new_through(origin, through).into()
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS intersection

#[derive(Debug, Eq, PartialEq)]
pub enum IHalfLineLineSegmentSoS {
  Crossing(Orientation),
}

// Line / LineSegment intersection.
// If the line is colinear with an endpoint in the linesegment, the endpoint
// will be considered to be to the left of the line.
impl<T> Intersects<LineSegmentView<'_, T>> for &HalfLineSoS<'_, T>
where
  T: PolygonScalar,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T>) -> Option<Self::Result> {
    let b1 = other.min.inner();
    let b2 = other.max.inner();

    let origin = &self.line.origin;

    let l1_to_b1 = Point::orient_along_direction(origin, self.line.direction, b1);
    let l1_to_b2 = Point::orient_along_direction(origin, self.line.direction, b2);

    use IHalfLineLineSegmentSoS::*;
    use Orientation::*;
    match (l1_to_b1, l1_to_b2) {
      // l1 and l2 are parallel and therefore cannot cross.
      (CoLinear, CoLinear) => None,
      // b1 is colinear with l1 and there is a crossing if we lean in the direction of b2
      (CoLinear, _) => {
        let l2_to_a1 = Point::orient(b1, b2, origin);
        if l1_to_b2 == l2_to_a1 {
          Some(Crossing(l1_to_b2))
        } else {
          None
        }
      }
      // b2 is colinear with l1 and there is a crossing if we lean in the direction of b1
      (_, CoLinear) => {
        let l2_to_a1 = Point::orient(b2, b1, origin);
        if l1_to_b1 == l2_to_a1 {
          Some(Crossing(l1_to_b1))
        } else {
          None
        }
      }
      _ => {
        if l1_to_b1 == l1_to_b2.reverse() {
          // b1 and b2 are on opposite sides of the line.
          // There is a crossing if l2 is on the origin side of l1.
          let l2_to_a1 = Point::orient(b1, b2, origin);
          if l1_to_b1 == l2_to_a1.reverse() {
            Some(Crossing(CoLinear))
          } else {
            None
          }
        } else {
          // b1 and b2 are on the same side of the line.
          // There is definitely no intersection.
          None
        }
      }
    }
  }
}

// impl<T> Intersects<&DirectedEdgeOwner<T, 2>> for &HalfLineSoS<T, 2>
// where
//   T: PolygonScalar,
// {
//   type Result = IHalfLineLineSegmentSoS;
//   fn intersect(self, other: &DirectedEdgeOwner<T, 2>) -> Option<Self::Result> {
//     let line: LineSegmentView<'_, T, 2> = other.into();
//     self.intersect(line)
//   }
// }

impl<T> Intersects<DirectedEdgeView<'_, T>> for &HalfLineSoS<'_, T>
where
  T: PolygonScalar,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: DirectedEdgeView<'_, T>) -> Option<Self::Result> {
    let line: LineSegmentView<'_, T> = other.into();
    self.intersect(line)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::{ILineSegment, LineSegment, Polygon};

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[test]
  fn ray_intersect_unit_1() {
    let line: LineSegment<i8> = LineSegment::from((-1, 127)..(-5, 48));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let origin = Point::new([79, 108]);
    let ray = HalfLineSoS::new_directed(&origin, &direction);

    assert_eq!(ray.intersect(line.as_ref()), None);
  }

  #[test]
  fn ray_intersect_unit_2() {
    let line: LineSegment<i8> = LineSegment::from((0, 0)..(-1, 127));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let origin = Point::new([79, 108]);
    let ray = HalfLineSoS::new_directed(&origin, &direction);

    assert_eq!(ray.intersect(line.as_ref()), None);
  }

  #[test]
  fn ray_intersect_unit_3() {
    let line1: LineSegment<i8> = LineSegment::from((0, 0)..(0, 1));
    let line2: LineSegment<i8> = LineSegment::from((0, -1)..(0, 0));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let origin = Point::new([-1, 0]);
    let ray = HalfLineSoS::new_directed(&origin, &direction);

    assert!(ray.intersect(line1.as_ref()).is_some());
    assert!(ray.intersect(line2.as_ref()).is_some());
  }

  #[test]
  fn ray_intersect_unit_4() {
    use crate::data::PointLocation;
    let poly = Polygon::new(vec![
      Point::new([0, -1]),
      Point::new([0, 0]),
      Point::new([0, 1]),
      Point::new([-10, 1]),
      Point::new([-10, -1]),
    ])
    .unwrap();

    let origin = Point::new([-1, 0]);
    assert_eq!(poly.locate(&origin), PointLocation::Inside);

    let origin = Point::new([0, 0]);
    assert_eq!(poly.locate(&origin), PointLocation::OnBoundary);

    let origin = Point::new([1, 0]);
    assert_eq!(poly.locate(&origin), PointLocation::Outside);
  }

  #[test]
  fn ray_intersect_unit_5() {
    let line: LineSegment<i8> = LineSegment::from((0, 0)..(0, 1));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let origin = Point::new([1, 0]);
    let ray = HalfLineSoS::new_directed(&origin, &direction);

    assert!(ray.intersect(line.as_ref()).is_none());
  }

  #[test]
  fn ray_intersect_unit_6() {
    let line: LineSegment<i8> = LineSegment::from((0, 0)..(0, 1));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let origin = Point::new([1, 0]);
    let ray = HalfLineSoS::new_directed(&origin, &direction);

    assert!(ray.intersect(line.as_ref()).is_none());
  }

  #[proptest]
  fn raw_intersection_count_prop(poly: Polygon<i8>, line: LineSoS<i8>) {
    let line: LineSoSView<'_, i8> = (&line).into();
    let mut intersections = 0;
    for edge in poly.iter_boundary_edges() {
      if line.intersect(edge).is_some() {
        intersections += 1;
      }
    }
    prop_assert_eq!(intersections % 2, 0);
  }

  #[proptest]
  fn overlapping_segments_have_intersection_point(line1: LineSegment<i8>, line2: LineSegment<i8>) {
    let overlapping = line1.intersect(&line2);

    if matches!(overlapping, Some(ILineSegment::Overlap(_))) {
      let line1: LineView<i8> = (&line1).into();
      let line2: LineView<i8> = (&line2).into();
      let intersection = line1.intersection_point(&line2);
      prop_assert!(intersection.is_some());
    }
  }
}
