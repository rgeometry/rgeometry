use std::ops::Sub;

use crate::array::raw_arr_zipwith;
use crate::vector::Vector;

impl<'a, 'b, T, const N: usize> Sub<&'a Vector<T, N>> for &'b Vector<T, N>
where
  T: Sub<T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn sub(self: &'b Vector<T, N>, other: &'a Vector<T, N>) -> Self::Output {
    // Vector(raw_arr_sub(&self.0, &other.0))
    Vector(raw_arr_zipwith(&self.0, &other.0, |a, b| {
      a.clone() - b.clone()
    }))
  }
}
