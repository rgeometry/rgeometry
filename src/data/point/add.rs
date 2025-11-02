use super::Point;
use super::Vector;
use array_init::array_init;
use num_traits::NumOps;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;

// &point + &vector = point
impl<'a, 'b, T, const N: usize> Add<&'a Vector<T, N>> for &'b Point<T, N>
where
  T: Add<Output = T> + Clone,
{
  type Output = Point<T, N>;

  fn add(self: &'b Point<T, N>, other: &'a Vector<T, N>) -> Self::Output {
    Point {
      array: array_init(|i| self.array.index(i).clone() + other.0.index(i).clone()),
    }
  }
}

// point + vector = point
impl<T, const N: usize> Add<Vector<T, N>> for Point<T, N>
where
  T: Add<Output = T> + Clone,
{
  type Output = Point<T, N>;

  fn add(self: Point<T, N>, other: Vector<T, N>) -> Self::Output {
    self.add(&other)
  }
}

// point + &vector = point
impl<T, const N: usize> Add<&Vector<T, N>> for Point<T, N>
where
  T: Add<Output = T> + Clone,
{
  type Output = Point<T, N>;

  fn add(self: Point<T, N>, other: &Vector<T, N>) -> Self::Output {
    (&self).add(other)
  }
}

// point += &vector
impl<T, const N: usize> AddAssign<&Vector<T, N>> for Point<T, N>
where
  T: NumOps + Clone + AddAssign,
{
  fn add_assign(&mut self, other: &Vector<T, N>) {
    for i in 0..N {
      self.array[i] += other.0.index(i).clone()
    }
  }
}

// point += vector
impl<T, const N: usize> AddAssign<Vector<T, N>> for Point<T, N>
where
  T: NumOps + Clone + AddAssign,
{
  fn add_assign(&mut self, other: Vector<T, N>) {
    self.add_assign(&other)
  }
}
