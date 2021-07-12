use super::DirectedEdge;
use super::LineSegmentSoS;
use super::LineSegmentView;
use super::Point;
use super::Vector;
use crate::data::point::PointSoS;
use crate::Intersects;
use crate::{Orientation, PolygonScalar, SoS};

use num::traits::*;
use std::cmp::Ordering;

///////////////////////////////////////////////////////////////////////////////
// Line

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Line<T, const N: usize> {
  pub origin: Point<T, N>,
  pub direction: Direction<T, N>,
}

#[derive(Debug, Clone)]
pub enum Direction<T, const N: usize> {
  Vector(Vector<T, N>),
  Through(Point<T, N>),
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS

#[derive(Debug, Clone)]
pub struct LineSoS<T, const N: usize> {
  pub origin: Point<T, N>,
  pub direction: Direction<T, N>,
}

impl<T, const N: usize> LineSoS<T, N> {
  pub fn new(origin: Point<T, N>, direction: Direction<T, N>) -> LineSoS<T, N> {
    LineSoS { origin, direction }
  }

  pub fn new_directed(origin: Point<T, N>, vector: Vector<T, N>) -> LineSoS<T, N> {
    LineSoS::new(origin, Direction::Vector(vector))
  }

  pub fn new_through(origin: Point<T, N>, through: Point<T, N>) -> LineSoS<T, N> {
    LineSoS::new(origin, Direction::Through(through))
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
impl<T> Intersects<LineSegmentView<'_, T, 2>> for &LineSoS<T, 2>
where
  T: PolygonScalar,
{
  type Result = ILineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T, 2>) -> Option<Self::Result> {
    let b1 = other.min.inner();
    let b2 = other.max.inner();
    let origin = &self.origin;
    let l1_to_b1;
    let l1_to_b2;
    match &self.direction {
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

///////////////////////////////////////////////////////////////////////////////
// Half-Line SoS

pub struct HalfLineSoS<T, const N: usize> {
  pub lean: SoS,
  pub line: LineSoS<T, N>,
}

impl<T, const N: usize> HalfLineSoS<T, N> {
  pub fn new(origin: Point<T, N>, direction: Direction<T, N>) -> HalfLineSoS<T, N> {
    HalfLineSoS {
      lean: SoS::CounterClockWise,
      line: LineSoS::new(origin, direction),
    }
  }

  pub fn new_directed(origin: Point<T, N>, vector: Vector<T, N>) -> HalfLineSoS<T, N> {
    HalfLineSoS {
      lean: SoS::CounterClockWise,
      line: LineSoS::new_directed(origin, vector),
    }
  }

  pub fn new_through(origin: Point<T, N>, through: Point<T, N>) -> HalfLineSoS<T, N> {
    HalfLineSoS {
      lean: SoS::CounterClockWise,
      line: LineSoS::new_through(origin, through),
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS intersection

#[derive(Debug, Eq, PartialEq)]
pub enum IHalfLineLineSegmentSoS {
  Crossing,
}

// Line / LineSegment intersection.
// If the line is colinear with an endpoint in the linesegment, the endpoint
// will be considered to be to the left of the line.
impl<T> Intersects<LineSegmentView<'_, T, 2>> for &HalfLineSoS<T, 2>
where
  T: PolygonScalar,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T, 2>) -> Option<Self::Result> {
    let b1 = other.min.inner();
    let b2 = other.max.inner();

    let origin = &self.line.origin;

    let l1_to_b1;
    let l1_to_b2;
    match &self.line.direction {
      Direction::Vector(direction) => {
        l1_to_b1 = Point::orient_along_vector(origin, direction, b1).sos(self.lean.reverse());
        l1_to_b2 = Point::orient_along_vector(origin, direction, b2).sos(self.lean.reverse());
      }
      Direction::Through(through) => {
        l1_to_b1 = Point::orient(origin, through, b1).sos(self.lean.reverse());
        l1_to_b2 = Point::orient(origin, through, b2).sos(self.lean.reverse());
      }
    }

    if l1_to_b1 == l1_to_b2.reverse() {
      // b1 and b2 are on opposite sides of the line.
      // FIXME: What is the right lean direction here?
      let l2_to_a1 = Point::orient(b1, b2, origin);
      if l1_to_b1.orient() == l2_to_a1.reverse() {
        Some(IHalfLineLineSegmentSoS::Crossing)
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

impl<T> Intersects<&DirectedEdge<T, 2>> for &HalfLineSoS<T, 2>
where
  T: PolygonScalar,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: &DirectedEdge<T, 2>) -> Option<Self::Result> {
    let line: LineSegmentView<'_, T, 2> = other.into();
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
    let line: LineSegment<i8, 2> = LineSegment::from((-1, 127)..(-5, 48));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let ray = HalfLineSoS {
      lean: SoS::CounterClockWise,
      line: LineSoS {
        origin: Point::new([79, 108]),
        direction: Direction::Vector(direction),
      },
    };

    assert_eq!(ray.intersect(line.as_ref()), None);
  }

  #[test]
  fn ray_intersect_unit_2() {
    let line: LineSegment<i8, 2> = LineSegment::from((0, 0)..(-1, 127));
    let direction: Vector<i8, 2> = Vector([1, 0]);
    let ray = HalfLineSoS {
      lean: SoS::CounterClockWise,
      line: LineSoS {
        origin: Point::new([79, 108]),
        direction: Direction::Vector(direction),
      },
    };

    assert_eq!(ray.intersect(line.as_ref()), None);
  }

  #[proptest]
  fn raw_intersection_count_prop(poly: Polygon<i8>, line: LineSoS<i8, 2>) {
    let mut intersections = 0;
    for edge in poly.iter_boundary_edges() {
      let edge: LineSegment<i8, 2> = edge.into();
      if line.intersect(edge.as_ref()).is_some() {
        intersections += 1;
      }
    }
    prop_assert_eq!(intersections % 2, 0);
  }
}
