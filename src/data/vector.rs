use array_init::{array_init, try_array_init};
use num_rational::BigRational;
use num_traits::{NumOps, Signed};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use crate::data::Point;
use crate::Extended;
use crate::Orientation;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Vector<T, const N: usize>(pub [T; N]);

#[derive(Debug, Clone, Copy)]
pub struct VectorView<'a, T, const N: usize>(pub &'a [T; N]);

impl<T, const N: usize> Distribution<Vector<T, N>> for Standard
where
  Standard: Distribution<T>,
{
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector<T, N> {
    Vector(array_init(|_| rng.gen()))
  }
}

impl<T, const N: usize> Vector<T, N>
where
  T: Clone,
{
  pub fn qda(&self) -> T {
    unimplemented!();
  }

  pub fn squared_magnitude(&self) -> T
  where
    T: Sum + AddAssign,
    for<'a> &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T>,
  {
    self.0.iter().map(|elt| elt * elt).sum()
  }

  pub fn map<U, F>(&self, f: F) -> Vector<U, N>
  where
    T: Clone,
    F: Fn(T) -> U,
  {
    Vector(array_init(|i| f(self.0[i].clone())))
  }

  pub fn cast<U>(&self) -> Vector<U, N>
  where
    T: Clone + Into<U>,
  {
    Vector(array_init(|i| self.0[i].clone().into()))
  }
}

impl<T, const N: usize> Index<usize> for Vector<T, N> {
  type Output = T;
  fn index(&self, index: usize) -> &T {
    self.0.index(index)
  }
}

impl<T, const N: usize> From<Point<T, N>> for Vector<T, N> {
  fn from(point: Point<T, N>) -> Vector<T, N> {
    Vector(point.array)
  }
}

impl<'a, T, const N: usize> From<&'a Point<T, N>> for &'a Vector<T, N> {
  fn from(point: &Point<T, N>) -> &Vector<T, N> {
    unsafe { &*(point as *const Point<T, N> as *const Vector<T, N>) }
  }
}

impl<'a, T, const N: usize> From<&'a Point<T, N>> for VectorView<'a, T, N> {
  fn from(point: &Point<T, N>) -> VectorView<'_, T, N> {
    VectorView(&point.array)
  }
}

impl<'a, const N: usize> TryFrom<Vector<f64, N>> for Vector<BigRational, N> {
  type Error = ();
  fn try_from(point: Vector<f64, N>) -> Result<Vector<BigRational, N>, ()> {
    Ok(Vector(try_array_init(|i| {
      BigRational::from_float(point[i]).ok_or(())
    })?))
  }
}

impl<T> Vector<T, 2> {
  // Unit vector pointing to the right.
  pub fn unit_right() -> Vector<T, 2>
  where
    T: Extended,
  {
    Vector([T::from_constant(1), T::from_constant(0)])
  }

  pub fn ccw_cmp_around(&self, p: &Vector<T, 2>, q: &Vector<T, 2>) -> Ordering
  where
    T: Extended,
  {
    self.ccw_cmp_around_with(&Vector([T::from_constant(1), T::from_constant(0)]), p, q)
  }
  pub fn ccw_cmp_around_with(
    &self,
    z: &Vector<T, 2>,
    p: &Vector<T, 2>,
    q: &Vector<T, 2>,
  ) -> Ordering
  where
    T: Extended,
  {
    Orientation::ccw_cmp_around_with(z, &self.0, &p.0, &q.0)
  }

  // FIXME: rename to sort_around_origin
  // FIXME: sort by magnitude if two vectors have the same angle.
  pub fn sort_around(pts: &mut Vec<Vector<T, 2>>)
  where
    T: Extended,
  {
    let origin = [T::from_constant(0), T::from_constant(0)];
    pts.sort_unstable_by(|a, b| {
      Orientation::ccw_cmp_around_with(
        &Vector([T::from_constant(1), T::from_constant(0)]),
        &origin,
        &a.0,
        &b.0,
      )
    })
    // L.sortBy (ccwCmpAround c <> cmpByDistanceTo c)
  }

  pub fn cmp_along(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: crate::PolygonScalar,
  {
    // Rotate the vector 90 degrees counterclockwise.
    match Orientation::along_perp_vector(&p.array, self, &q.array) {
      Orientation::CounterClockWise => Ordering::Greater,
      Orientation::ClockWise => Ordering::Less,
      Orientation::CoLinear => Ordering::Equal,
    }
  }
}

mod add;
mod div;
mod mul;
mod sub;

impl<T, const N: usize> Sum for Vector<T, N>
where
  T: NumOps + AddAssign + Clone + Sum,
  // for<'c> &'c T: Add<&'c T, Output = T>,
{
  fn sum<I>(iter: I) -> Vector<T, N>
  where
    I: Iterator<Item = Vector<T, N>>,
  {
    let mut acc = Vector(array_init(|_| std::iter::empty().sum()));
    for vec in iter {
      acc += vec;
    }
    acc
  }
}

impl<T, const N: usize> Neg for Vector<T, N>
where
  T: NumOps + Neg<Output = T> + Clone,
{
  type Output = Self;
  fn neg(self) -> Self {
    Vector(array_init(|i| self.0.index(i).clone().neg()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing::*;

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[test]
  fn unit_1() {
    let v = Vector([72i8, -113]);
    let p1 = Point { array: [-84, 93] };
    let p2 = Point { array: [114, -71] };
    v.cmp_along(&p1, &p2);
  }

  #[proptest]
  fn cmp_along_fuzz(v: Vector<i8, 2>, p1: Point<i8, 2>, p2: Point<i8, 2>) {
    v.cmp_along(&p1, &p2);
  }

  #[proptest]
  fn cmp_along_prop_x(p1: Point<i8, 2>, p2: Point<i8, 2>) {
    let v = Vector([1, 0]);
    prop_assert_eq!(v.cmp_along(&p1, &p2), p1.x_coord().cmp(p2.x_coord()));
  }

  #[proptest]
  fn cmp_along_prop_y(p1: Point<i8, 2>, p2: Point<i8, 2>) {
    let v = Vector([0, 1]);
    prop_assert_eq!(v.cmp_along(&p1, &p2), p1.y_coord().cmp(p2.y_coord()));
  }
}
