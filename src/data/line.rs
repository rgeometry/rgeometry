use super::LineSegmentSoS;
use super::LineSegmentView;
use super::Point;
use super::Vector;
use crate::data::point::PointSoS;
use crate::Intersects;
use crate::{Orientation, PolygonScalar};

use num::traits::*;
use std::cmp::Ordering;

///////////////////////////////////////////////////////////////////////////////
// Line

pub enum Line<T, const N: usize> {
  Line {
    origin: Point<T, N>,
    direction: Vector<T, N>,
  },
  LineThrough {
    origin: Point<T, N>,
    through: Point<T, N>,
  },
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS

pub enum LineSoS<T, const N: usize> {
  Line {
    origin: Point<T, N>,
    direction: Vector<T, N>,
  },
  LineThrough {
    origin: Point<T, N>,
    through: Point<T, N>,
  },
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
  T: Clone + NumOps<T, T> + Ord + std::fmt::Debug + crate::Extended,
{
  type Result = ILineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T, 2>) -> Option<Self::Result> {
    match self {
      LineSoS::Line { origin, direction } => {
        let b1 = other.min.inner();
        let b2 = other.max.inner();
        let l1_to_b1 =
          Point::orient_along_vector(origin, direction, b1).then(Orientation::CounterClockWise);
        let l1_to_b2 =
          Point::orient_along_vector(origin, direction, b2).then(Orientation::CounterClockWise);
        if l1_to_b1 == l1_to_b2.reverse() {
          Some(ILineLineSegmentSoS::Crossing)
        } else {
          None
        }
      }
      LineSoS::LineThrough { origin, through } => {
        let b1 = other.min.inner();
        let b2 = other.max.inner();
        let l1_to_b1 = Point::orient(origin, through, b1).then(Orientation::CounterClockWise);
        let l1_to_b2 = Point::orient(origin, through, b2).then(Orientation::CounterClockWise);
        if l1_to_b1 == l1_to_b2.reverse() {
          Some(ILineLineSegmentSoS::Crossing)
        } else {
          None
        }
      }
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Half-Line SoS

pub struct HalfLineSoS<T, const N: usize> {
  line: LineSoS<T, N>,
}

///////////////////////////////////////////////////////////////////////////////
// Line SoS intersection

pub enum IHalfLineLineSegmentSoS {
  Crossing,
}

// Line / LineSegment intersection.
// If the line is colinear with an endpoint in the linesegment, the endpoint
// will be considered to be to the left of the line.
impl<T> Intersects<LineSegmentView<'_, T, 2>> for &HalfLineSoS<T, 2>
where
  T: Clone + PolygonScalar + Ord + std::fmt::Debug + crate::Extended,
{
  type Result = IHalfLineLineSegmentSoS;
  fn intersect(self, other: LineSegmentView<'_, T, 2>) -> Option<Self::Result> {
    let ILineLineSegmentSoS::Crossing = self.line.intersect(other)?;
    match &self.line {
      LineSoS::Line { origin, direction } => {
        let mut b1 = other.min.inner();
        let mut b2 = other.max.inner();
        if direction.cmp_along(b1, b2) == Ordering::Greater {
          std::mem::swap(&mut b1, &mut b2);
        }
        if Point::orient(b1, b2, origin) == Orientation::ClockWise {
          Some(IHalfLineLineSegmentSoS::Crossing)
        } else {
          None
        }
      }
      LineSoS::LineThrough { origin, through } => {
        let b1 = other.min.inner();
        let b2 = other.max.inner();
        let l1_to_b1 = Point::orient(origin, through, b1).then(Orientation::CounterClockWise);
        let l1_to_b2 = Point::orient(origin, through, b2).then(Orientation::CounterClockWise);
        let l2_to_a1 = Point::orient(b1, b2, origin).then(Orientation::CounterClockWise);
        let l2_to_a2 = Point::orient(b1, b2, through).then(Orientation::CounterClockWise);
        // l1_to_b1 != l1_to_b2, this is checked earlier.
        // if l1_to_b1 == l1_to_b2 {
        //   return None; // b1 and b2 are on the same side of the line, no crossing.
        // }
        // If HalfLineSegment crosses origin->through, return Some(Crossing)
        if l2_to_a1 == l2_to_a2.reverse() {
          // the line-segment crosses the line between origin and through.
          return Some(IHalfLineLineSegmentSoS::Crossing);
        }
        if l2_to_a2 == l1_to_b1.reverse() {
          // either: origin -> through -> line-segment
          // or:     line-segment <- through <- origin
          return Some(IHalfLineLineSegmentSoS::Crossing);
        }
        return None;
        // If HalfLineSegment is on the left of through, check that b1->origin->through is CCW.
        // If so, return Some(Crossing)
        // If HalfLineSegment is on the right of through, check that b1->origin->through is CW.
        // If so, return Some(Crossing)
        // Otherwise, return None.
      }
    }
  }
}
