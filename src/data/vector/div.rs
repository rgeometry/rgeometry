use array_init::array_init;
use num_traits::NumOps;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;

use super::Vector;
use super::VectorView;

impl<T, const N: usize> Div<T> for Vector<T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;

  fn div(self: Vector<T, N>, other: T) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() / other.clone()))
  }
}

impl<'a, T, const N: usize> Div<T> for VectorView<'a, T, N>
where
  T: NumOps + Clone,
{
  type Output = Vector<T, N>;
  fn div(self, other: T) -> Vector<T, N> {
    Vector(array_init(|i| self.0[i].clone() / other.clone()))
  }
}
