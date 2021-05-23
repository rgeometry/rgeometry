use super::{Point, PointLocation};
use crate::{Error, Orientation, PolygonScalar, PolygonScalarRef};
use claim::debug_assert_ok;
use std::ops::*;

// FIXME: Support n-dimensional triangles?
pub struct Triangle<T>([Point<T, 2>; 3]);

impl<T> Triangle<T>
where
  T: PolygonScalar,
  for<'a> &'a T: PolygonScalarRef<&'a T, T>,
{
  pub fn new(pts: [Point<T, 2>; 3]) -> Triangle<T> {
    let triangle = Triangle(pts);
    debug_assert_ok!(triangle.validate());
    triangle
  }

  pub fn validate(&self) -> Result<(), Error> {
    self.view().validate()
  }

  // O(1)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    self.view().locate(pt)
  }

  pub fn view(&'_ self) -> TriangleView<'_, T> {
    TriangleView([&self.0[0], &self.0[1], &self.0[2]])
  }
}

pub struct TriangleView<'a, T>([&'a Point<T, 2>; 3]);

impl<'a, T> TriangleView<'a, T>
where
  T: PolygonScalar,
  // for<'x> &'x T: PolygonScalarRef<&'x T, T>,
{
  // O(1)
  pub fn new(pts: [&'a Point<T, 2>; 3]) -> TriangleView<'a, T> {
    let triangle = TriangleView(pts);
    debug_assert_ok!(triangle.validate());
    triangle
  }

  // O(1)
  pub fn validate(&self) -> Result<(), Error> {
    let arr = &self.0;
    if arr.index(0).orientation(&arr[1], &arr[2]) != Orientation::CounterClockWise {
      Err(Error::ClockWiseViolation)
    } else {
      Ok(())
    }
  }

  // O(1)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    use Orientation::*;
    debug_assert_ok!(self.validate());
    let [a, b, c] = self.0;
    let ab = a.orientation(b, pt);
    let bc = b.orientation(c, pt);
    let ca = c.orientation(a, pt);
    if ab == ClockWise || bc == ClockWise || ca == ClockWise {
      PointLocation::Outside
    } else if ab == CoLinear || bc == CoLinear || ca == CoLinear {
      PointLocation::OnBoundary
    } else {
      PointLocation::Inside
    }
  }
}
