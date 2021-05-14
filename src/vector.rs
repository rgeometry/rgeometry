use array_init::array_init;
use num_traits::identities::Zero;
use num_traits::Num;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use super::array::*;
use super::point::Point;

#[derive(Debug, Clone, Copy)]
pub struct Vector<T, const N: usize>(pub [T; N]);

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
  pub fn to_point(&self) -> Point<T, N> {
    Point(self.clone().0)
  }
  pub fn qda(&self) -> T {
    unimplemented!();
  }
}

impl<T, const N: usize> From<Point<T, N>> for Vector<T, N> {
  fn from(point: Point<T, N>) -> Vector<T, N> {
    Vector(point.0)
  }
}

impl<T> Vector<T, 2> {
  pub fn ccw_cmp_around(&self, p: &Vector<T, 2>, q: &Vector<T, 2>) -> Ordering
  where
    T: Num + Clone + PartialOrd,
    for<'a> &'a T: Neg<Output = T> + Sub<Output = T> + Mul<Output = T>,
  {
    self.ccw_cmp_around_with(&Vector([T::one(), T::zero()]), p, q)
  }
  pub fn ccw_cmp_around_with(
    &self,
    z: &Vector<T, 2>,
    p: &Vector<T, 2>,
    q: &Vector<T, 2>,
  ) -> Ordering
  where
    T: Num + Clone + PartialOrd,
    for<'a> &'a T: Sub<Output = T> + Mul<Output = T> + Neg<Output = T>,
  {
    ccw_cmp_around_origin_with(&z.0, &(p - self).0, &(q - self).0)
  }

  pub fn sort_around(pts: &mut Vec<Vector<T, 2>>)
  where
    T: Num + PartialOrd + Clone,
    for<'a> &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T> + Neg<Output = T>,
  {
    pts.sort_unstable_by(|a, b| ccw_cmp_around_origin_with(&[T::one(), T::zero()], &a.0, &b.0))
    // unimplemented!();
    // L.sortBy (ccwCmpAround c <> cmpByDistanceTo c)
  }
}

mod add;
mod sub;

impl<T, const N: usize> Zero for Vector<T, N>
where
  T: Zero + Clone + Add + Add<Output = T>,
{
  fn zero() -> Vector<T, N> {
    Vector(array_init(|_| Zero::zero()))
  }
  fn is_zero(&self) -> bool {
    self.0.iter().all(Zero::is_zero)
  }
}
