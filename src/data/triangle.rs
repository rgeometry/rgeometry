use super::{Point, PointLocation};
use crate::{Error, Orientation, PolygonScalar, PolygonScalarRef};
use claim::debug_assert_ok;
use num_traits::*;
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

  pub fn new_unchecked(pts: [&'a Point<T, 2>; 3]) -> TriangleView<'a, T> {
    TriangleView(pts)
  }

  // O(1)
  pub fn validate(&self) -> Result<(), Error> {
    if self.orientation() != Orientation::CounterClockWise {
      Err(Error::ClockWiseViolation)
    } else {
      Ok(())
    }
  }

  pub fn orientation(&self) -> Orientation {
    let arr = &self.0;
    Orientation::new(&arr[0], &arr[1], &arr[2])
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

  pub fn signed_area<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + FromPrimitive + Clone,
  {
    self.signed_area_2x::<F>() / F::from_usize(2).unwrap()
  }

  pub fn signed_area_2x<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + Clone,
  {
    let [a, b, c] = self.0;
    let ax = a.x_coord().clone().into();
    let ay = a.y_coord().clone().into();
    let bx = b.x_coord().clone().into();
    let by = b.y_coord().clone().into();
    let cx = c.x_coord().clone().into();
    let cy = c.y_coord().clone().into();
    ax.clone() * by.clone() - bx.clone() * ay.clone() + bx * cy.clone() - cx.clone() * by + cx * ay
      - ax * cy
    // x1*y2 - x2*y1 +
    // x2*y3 - x3*y2 +
    // x3*y1 - x1*y3
  }
}
