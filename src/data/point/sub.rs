use array_init::array_init;
use num_traits::RefNum;
use std::ops::Index;
use std::ops::Sub;

use super::Point;
use super::Vector;
use crate::array::*;

// Compiler bug: https://github.com/rust-lang/rust/issues/39959
// When the bug has been fixed then we can get rid of the 'Clone' requirement.
// point - point = vector
impl<'a, 'b, T, const N: usize> Sub<&'a Point<T, N>> for &'b Point<T, N>
where
  // T: Sub<T, Output = T> + Clone,
  T: Sub<T, Output = T> + Clone,
  // for<'c> &'c T: Sub<&'c T, Output = T> + Clone,
  // for<'c> &'c T: RefNum<T>,
{
  type Output = Vector<T, N>;

  fn sub(self: &'b Point<T, N>, other: &'a Point<T, N>) -> Self::Output {
    // Vector(raw_arr_sub(&self.array, &other.array))
    Vector(array_init(|i| {
      self.array.index(i).clone() - other.array.index(i).clone()
    }))
  }
}

impl<T, const N: usize> Sub<Point<T, N>> for Point<T, N>
where
  // T: Sub<T, Output = T> + Clone,
  T: Sub<T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn sub(self: Point<T, N>, other: Point<T, N>) -> Self::Output {
    Sub::sub(&self, &other)
  }
}
