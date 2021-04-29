use std::ops::Sub;

use crate::array::raw_arr_zipwith;
use crate::point::Point;
use crate::vector::Vector;

// Compiler bug: https://github.com/rust-lang/rust/issues/39959
// When the bug has been fixed then we can get rid of the 'Clone' requirement.
// point - point = vector
impl<'a, 'b, T, const N: usize> Sub<&'a Point<T, N>> for &'b Point<T, N>
where
  T: Sub<T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn sub(self: &'b Point<T, N>, other: &'a Point<T, N>) -> Self::Output {
    // Vector(raw_arr_sub(&self.0, &other.0))
    Vector(raw_arr_zipwith(&self.0, &other.0, |a, b| {
      a.clone() - b.clone()
    }))
  }
}
