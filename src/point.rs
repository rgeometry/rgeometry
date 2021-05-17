use array_init::array_init;
use num_traits::Num;
use num_traits::Zero;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
// use std::ops::Add;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use super::array::*;
use super::vector::{Vector, VectorView};

#[derive(Debug, Clone, Copy)]
pub struct Point<T, const N: usize> {
  pub array: [T; N],
}

// Random sampling.
impl<T, const N: usize> Distribution<Point<T, N>> for Standard
where
  Standard: Distribution<T>,
{
  // FIXME: Unify with code for Vector.
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point<T, N> {
    Point {
      array: array_init(|_| rng.gen()),
    }
  }
}

// Methods on N-dimensional points.
impl<T: Clone, const N: usize> Point<T, N> {
  pub fn new(array: [T; N]) -> Point<T, N> {
    Point { array: array }
  }
  pub fn cmp_distance_to(&self, p: &Point<T, N>, q: &Point<T, N>) -> Ordering
  where
    T: Zero + PartialOrd,
    for<'a> &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T>,
  {
    self
      .squared_euclidean_distance(p)
      .partial_cmp(&self.squared_euclidean_distance(q))
      .unwrap_or(Ordering::Equal)
  }
  pub fn squared_euclidean_distance(&self, rhs: &Point<T, N>) -> T
  where
    T: Zero,
    for<'a> &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T>,
  {
    self
      .array
      .iter()
      .zip(rhs.array.iter())
      .fold(T::zero(), |sum, (a, b)| {
        let diff = a - b;
        sum + &diff * &diff
      })
  }

  // Similar to num_traits::identities::Zero but doesn't require an Add impl.
  pub fn zero() -> Self
  where
    T: Zero,
  {
    Point {
      array: array_init(|_| Zero::zero()),
    }
  }

  pub fn cast<U>(&self) -> Point<U, N>
  where
    U: From<T>,
  {
    Point {
      array: array_init(|i| self.array[i].clone().into()),
    }
  }
}

impl<T, const N: usize> Index<usize> for Point<T, N> {
  type Output = T;
  fn index(&self, key: usize) -> &T {
    self.array.index(key)
  }
}

impl<T, const N: usize> From<Vector<T, N>> for Point<T, N> {
  fn from(vector: Vector<T, N>) -> Point<T, N> {
    Point { array: vector.0 }
  }
}

// Methods on two-dimensional points.
impl<T> Point<T, 2> {
  pub fn turn(&self, q: &Point<T, 2>, r: &Point<T, 2>) -> Turn
  where
    T: Sub<T, Output = T> + Clone + Mul<T, Output = T> + PartialOrd,
    for<'a> &'a T: Sub<Output = T>,
  {
    raw_arr_turn(&self.array, &q.array, &r.array)
  }
  /// Docs?
  pub fn ccw_cmp_around(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Num + Clone + PartialOrd + Neg<Output = T>,
    for<'a> &'a T: Neg<Output = T> + Sub<Output = T> + Mul<Output = T>,
  {
    self.ccw_cmp_around_with(&Vector([T::one(), T::zero()]), p, q)
  }
  pub fn ccw_cmp_around_with(&self, z: &Vector<T, 2>, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Num + Clone + PartialOrd + Neg<Output = T>,
    for<'a> &'a T: Sub<Output = T> + Mul<Output = T> + Neg<Output = T>,
  {
    ccw_cmp_around_origin_with(&z.0, &(p - self).0, &(q - self).0)
  }
}

mod add;
mod sub;

// // Sigh, should relax Copy to Clone.
// impl<T, const N: usize> Zero for Point<T, N>
// where
//   T: Zero + Copy + Add + Add<Output = T>,
// {
//   fn zero() -> Point<T, N> {
//     Point([Zero::zero(); N])
//   }
//   fn is_zero(&self) -> bool {
//     self.0.iter().all(Zero::is_zero)
//   }
// }
