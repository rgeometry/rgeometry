use array_init::array_init;
use std::ops::Sub;

use crate::point::Point;
use crate::vector::Vector;

// Compiler bug: https://github.com/rust-lang/rust/issues/39959
// When the bug has been fixed then we can get rid of the 'Clone' requirement.
// point - point = vector
impl<'a, 'b, T, const N: usize> Sub<&'a Point<T, N>> for &'b Point<T, N>
where
  // T: Sub<T, Output = T> + Clone,
  T: Sub<T, Output = T> + Clone,
  // for<'c> &'c T: Sub<&'c T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn sub(self: &'b Point<T, N>, other: &'a Point<T, N>) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() - other.0[i].clone()))
  }
}
