use array_init::array_init;
use num_traits::NumOps;
use std::ops::Add;
use std::ops::Mul;

use crate::vector::Vector;
use crate::vector::VectorView;

impl<T, const N: usize> Mul<T> for Vector<T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;

  fn mul(self: Vector<T, N>, other: T) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() * other.clone()))
  }
}

impl<'a, T, const N: usize> Mul<T> for VectorView<'a, T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;
  fn mul(self, other: T) -> Vector<T, N> {
    Vector(array_init(|i| self.0[i].clone() * other.clone()))
  }
}
