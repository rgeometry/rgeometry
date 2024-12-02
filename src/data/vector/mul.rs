use array_init::array_init;
use num_traits::NumOps;
use std::ops::Mul;

use super::Vector;
use super::VectorView;

impl<T, const N: usize> Mul<T> for Vector<T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;

  fn mul(self: Vector<T, N>, other: T) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() * other.clone()))
  }
}

impl<T, const N: usize> Mul<T> for VectorView<'_, T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;
  fn mul(self, other: T) -> Vector<T, N> {
    Vector(array_init(|i| self.0[i].clone() * other.clone()))
  }
}
