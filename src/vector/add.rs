use array_init::array_init;
use num_traits::Num;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;

use crate::vector::Vector;
use crate::vector::VectorView;

impl<T, const N: usize> Add<Vector<T, N>> for Vector<T, N>
where
  T: Num + Clone,
{
  type Output = Vector<T, N>;

  fn add(self: Vector<T, N>, other: Vector<T, N>) -> Self::Output {
    Vector(array_init(|i| self.0[i].clone() + other.0[i].clone()))
  }
}

impl<T, const N: usize> AddAssign<Vector<T, N>> for Vector<T, N>
where
  T: Num + Clone + AddAssign,
{
  fn add_assign(&mut self, other: Vector<T, N>) {
    for i in 0..N {
      self.0[i] += other.0.index(i).clone()
    }
  }
}

impl<'a, T, const N: usize> Add for VectorView<'a, T, N>
where
  T: Num + Clone,
{
  type Output = Vector<T, N>;
  fn add(self, other: VectorView<'a, T, N>) -> Vector<T, N> {
    Vector(array_init(|i| self.0[i].clone() + other.0[i].clone()))
  }
}
