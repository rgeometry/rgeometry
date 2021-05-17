use array_init::array_init;
use num_traits::identities::Zero;
use num_traits::Num;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use super::array::*;
use super::point::Point;

#[derive(Debug, Clone)]
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

impl<'a, T, const N: usize> From<&'a Point<T, N>> for VectorView<'a, T, N> {
  fn from(point: &Point<T, N>) -> VectorView<'_, T, N> {
    VectorView(&point.array)
  }
}

impl<T> Vector<T, 2> {
  pub fn ccw_cmp_around(&self, p: &Vector<T, 2>, q: &Vector<T, 2>) -> Ordering
  where
    T: Num + Clone + PartialOrd + Neg<Output = T>,
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
    T: Num + Clone + PartialOrd + Neg<Output = T>,
    for<'a> &'a T: Sub<Output = T> + Mul<Output = T>,
  {
    ccw_cmp_around_origin_with(&z.0, &(p - self).0, &(q - self).0)
  }

  pub fn sort_around(pts: &mut Vec<Vector<T, 2>>)
  where
    T: Num + PartialOrd + Clone + Neg<Output = T>,
    for<'a> &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T>,
  {
    pts.sort_unstable_by(|a, b| ccw_cmp_around_origin_with(&[T::one(), T::zero()], &a.0, &b.0))
    // unimplemented!();
    // L.sortBy (ccwCmpAround c <> cmpByDistanceTo c)
  }
}

mod add;
mod div;
mod mul;
mod sub;

impl<T, const N: usize> Zero for Vector<T, N>
where
  T: Num + Clone,
{
  fn zero() -> Vector<T, N> {
    Vector(array_init(|_| Zero::zero()))
  }
  fn is_zero(&self) -> bool {
    self.0.iter().all(Zero::is_zero)
  }
}

impl<T, const N: usize> Sum for Vector<T, N>
where
  T: Num + AddAssign + Clone,
{
  fn sum<I>(iter: I) -> Vector<T, N>
  where
    I: Iterator<Item = Vector<T, N>>,
  {
    let mut acc = Zero::zero();
    for vec in iter {
      acc += vec;
    }
    acc
  }
}

impl<T, const N: usize> Neg for Vector<T, N>
where
  T: Num + Neg<Output = T> + Clone,
{
  type Output = Self;
  fn neg(self) -> Self {
    Vector(array_init(|i| self.0.index(i).clone().neg()))
  }
}
