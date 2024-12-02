use super::DirectedEdge;
use super::LineSegmentView;
use super::Point;
use super::Vector;
use crate::Intersects;
use crate::{Orientation, PolygonScalar};

///////////////////////////////////////////////////////////////////////////////
// Line

#[derive(Debug)]
pub struct Line<'a, T, const N: usize = 2> {
  pub origin: &'a Point<T, N>,
  pub direction: Direction<'a, T, N>,
}

impl<T, const N: usize> Copy for Line<'_, T, N> {}
impl<T, const N: usize> Clone for Line<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, T, const N: usize> Line<'a, T, N> {
  pub fn new(origin: &'a Point<T, N>, direction: Direction<'a, T, N>) -> Line<'a, T, N> {
    Line { origin, direction }
  }

  pub fn new_directed(origin: &'a Point<T, N>, vector: &'a Vector<T, N>) -> Line<'a, T, N> {
    Line::new(origin, Direction::Vector(vector))
  }

  pub fn new_through(origin: &'a Point<T, N>, through: &'a Point<T, N>) -> Line<'a, T, N> {
    Line::new(origin, Direction::Through(through))
  }
}

impl<T: PolygonScalar> Line<'_, T> {
  pub fn intersection_point(&self, other: &Self) -> Option<Point<T>> {
    let [x1, y1] = self.origin.array.clone();
    let [x2, y2] = match &self.direction {
      Direction::Through(pt) => pt.array.clone(),
      _ => unimplemented!(),
    };
    let [x3, y3] = other.origin.array.clone();
    let [x4, y4] = match &other.direction {
      Direction::Through(pt) => pt.array.clone(),
      _ => unimplemented!(),
    };
    let denom: T = (x1.clone() - x2.clone()) * (y3.clone() - y4.clone())
      - (y1.clone() - y2.clone()) * (x3.clone() - x4.clone());
    if denom == T::from_constant(0) {
      return None;
    }
    let part_a = x1.clone() * y2.clone() - y1.clone() * x2.clone();
    let part_b = x3.clone() * y4.clone() - y3.clone() * x4.clone();
    let x_num = part_a.clone() * (x3 - x4) - (x1 - x2) * part_b.clone();
    let y_num = part_a * (y3 - y4) - (y1 - y2) * part_b;
    Some(Point::new([x_num / denom.clone(), y_num / denom]))
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line (owned)

#[derive(Debug, Clone)]
pub struct Line_<T, const N: usize> {
  pub origin: Point<T, N>,
  pub direction: Direction_<T, N>,
}

impl<'a, T, const N: usize> From<&'a Line_<T, N>> for Line<'a, T, N> {
  fn from(line: &'a Line_<T, N>) -> Line<'a, T, N> {
    Line {
      origin: &line.origin,
      direction: (&line.direction).into(),
    }
  }
}

// impl<'a, T: PolygonScalar> Line<'a, T, 2> {
//   pub fn intersection_point(&self, other: &Self) -> Option<Point<T, 2>> {

//   }
// }

///////////////////////////////////////////////////////////////////////////////
// Direction

#[derive(Debug)]
pub enum Direction<'a, T, const N: usize> {
  Vector(&'a Vector<T, N>),
  Through(&'a Point<T, N>),
}

impl<T, const N: usize> Copy for Direction<'_, T, N> {}
impl<T, const N: usize> Clone for Direction<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

///////////////////////////////////////////////////////////////////////////////
// Direction_ (owned)

#[derive(Debug, Clone)]
pub enum Direction_<T, const N: usize> {
  Vector(Vector<T, N>),
  Through(Point<T, N>),
}

impl<'a, T, const N: usize> From<&'a Direction_<T, N>> for Direction<'a, T, N> {
  fn from(owned: &'a Direction_<T, N>) -> Direction<'a, T, N> {
    match owned {
      Direction_::Vector(v) => Direction::Vector(v),
      Direction_::Through(p) => Direction::Through(p),
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS

#[derive(Debug, Clone)]
pub struct LineSoS<'a, T, const N: usize = 2> {
  line: Line<'a, T, N>,
}

impl<'a, T, const N: usize> From<LineSoS<'a, T, N>> for Line<'a, T, N> {
  fn from(sos: LineSoS<'a, T, N>) -> Line<'a, T, N> {
    sos.line
  }
}

impl<'a, T, const N: usize> From<Line<'a, T, N>> for LineSoS<'a, T, N> {
  fn from(line: Line<'a, T, N>) -> LineSoS<'a, T, N> {
    LineSoS { line }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS Owned

#[derive(Debug, Clone)]
pub struct LineSoS_<T, const N: usize = 2> {
  line: Line_<T, N>,
}

impl<'a, T, const N: usize> From<&'a LineSoS_<T, N>> for LineSoS<'a, T, N> {
  fn from(line: &'a LineSoS_<T, N>) -> LineSoS<'a, T, N> {
    LineSoS {
      line: Line::from(&line.line),
    }
  }
}

impl<T, const N: usize> From<Line_<T, N>> for LineSoS_<T, N> {
  fn from(line: Line_<T, N>) -> LineSoS_<T, N> {
    LineSoS_ { line }
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
impl<T> Intersects<LineSegmentView<'_, T>> for &LineSoS<'_, T>
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
      Direction::Vector(direction) => {
        l1_to_b1 =
          Point::orient_along_vector(origin, direction, b1).then(Orientation::CounterClockWise);
        l1_to_b2 =
          Point::orient_along_vector(origin, direction, b2).then(Orientation::CounterClockWise);
      }
      Direction::Through(through) => {
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

impl<T> Intersects<DirectedEdge<'_, T>> for &LineSoS<'_, T>
where
  T: PolygonScalar,
{
  type Result = ILineLineSegmentSoS;
  fn intersect(self, other: DirectedEdge<'_, T>) -> Option<Self::Result> {
    let line_segment: LineSegmentView<'_, T> = other.into();
    self.intersect(line_segment)
  }
}

///////////////////////////////////////////////////////////////////////////////
// Half-Line SoS

pub struct HalfLineSoS<'a, T, const N: usize = 2> {
  line: Line<'a, T, N>,
}

impl<T, const N: usize> Copy for HalfLineSoS<'_, T, N> {}
impl<T, const N: usize> Clone for HalfLineSoS<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, T, const N: usize> From<LineSoS<'a, T, N>> for HalfLineSoS<'a, T, N> {
  fn from(sos: LineSoS<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    HalfLineSoS { line: sos.into() }
  }
}

impl<'a, T, const N: usize> From<Line<'a, T, N>> for HalfLineSoS<'a, T, N> {
  fn from(line: Line<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    HalfLineSoS { line }
  }
}

impl<'a, T, const N: usize> From<HalfLineSoS<'a, T, N>> for LineSoS<'a, T, N> {
  fn from(ray: HalfLineSoS<'a, T, N>) -> LineSoS<'a, T, N> {
    ray.line.into()
  }
}

impl<'a, T, const N: usize> From<HalfLineSoS<'a, T, N>> for Line<'a, T, N> {
  fn from(ray: HalfLineSoS<'a, T, N>) -> Line<'a, T, N> {
    ray.line
  }
}

impl<'a, T, const N: usize> HalfLineSoS<'a, T, N> {
  pub fn new(origin: &'a Point<T, N>, direction: Direction<'a, T, N>) -> HalfLineSoS<'a, T, N> {
    Line::new(origin, direction).into()
  }

  pub fn new_directed(origin: &'a Point<T, N>, vector: &'a Vector<T, N>) -> HalfLineSoS<'a, T, N> {
    Line::new_directed(origin, vector).into()
  }

  pub fn new_through(origin: &'a Point<T, N>, through: &'a Point<T, N>) -> HalfLineSoS<'a, T, N> {
    Line::new_through(origin, through).into()
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

impl<T> Intersects<DirectedEdge<'_, T>> for &HalfLineSoS<'_, T>
where
  T: PolygonScalar,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: DirectedEdge<'_, T>) -> Option<Self::Result> {
    let line: LineSegmentView<'_, T> = other.into();
    self.intersect(line)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::{LineSegment, Polygon};

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
  fn raw_intersection_count_prop(poly: Polygon<i8>, line: LineSoS_<i8>) {
    let line: LineSoS<'_, i8> = (&line).into();
    let mut intersections = 0;
    for edge in poly.iter_boundary_edges() {
      if line.intersect(edge).is_some() {
        intersections += 1;
      }
    }
    prop_assert_eq!(intersections % 2, 0);
  }
}
