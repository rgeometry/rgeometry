use array_init::array_init;
use std::ops::Index;
use std::ops::Sub;

use crate::vector::Vector;

impl<'a, 'b, T, const N: usize> Sub<&'a Vector<T, N>> for &'b Vector<T, N>
where
  T: Sub<T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn sub(self: &'b Vector<T, N>, other: &'a Vector<T, N>) -> Self::Output {
    Vector(array_init(|i| {
      self.0.index(i).clone() - other.0.index(i).clone()
    }))
  }
}
