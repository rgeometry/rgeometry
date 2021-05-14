use array_init::array_init;
use std::ops::Add;

use crate::vector::Vector;

impl<T, const N: usize> Add<Vector<T, N>> for Vector<T, N>
where
  T: Add<T, Output = T> + Clone,
{
  type Output = Vector<T, N>;

  fn add(self: Vector<T, N>, other: Vector<T, N>) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() + other.0[i].clone()))
  }
}
