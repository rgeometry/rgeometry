#[allow(unused_imports)]
use array_init::array_init;
use num_traits::identities::One;
use num_traits::identities::Zero;
use std::ops::Div;
use std::ops::Mul;

use crate::data::Point;
use crate::data::Polygon;
use crate::data::Vector;
use crate::matrix::{Matrix, MatrixMul};

pub trait TransformScalar: One + Zero + Div<Output = Self> + MatrixMul {}
impl<T> TransformScalar for T where T: One + Zero + Div<Output = Self> + MatrixMul {}

// Sigh, can't use nalgebra::Transform because it requires RealField + Copy.
// ndarray also requires Copy.
// Use const generics once const_generics and const_evaluatable_checked are stable.
#[derive(Clone, Debug)]
pub struct Transform<T, const N: usize>(Matrix<T>);

impl<T, const N: usize> Transform<T, N>
where
  T: TransformScalar,
{
  fn new(m: Matrix<T>) -> Transform<T, N> {
    assert_eq!(m.ncols(), N + 1);
    assert_eq!(m.nrows(), N + 1);
    Transform(m)
  }
  pub fn translate(vec: Vector<T, N>) -> Transform<T, N> {
    let mut m = Matrix::new(N + 1, N + 1);
    for i in 0..N {
      m[(i, i)] = T::one();
      m[(i, N)] = vec[i].clone();
    }
    m[(N, N)] = T::one();
    Transform::new(m)
  }

  pub fn scale(vec: Vector<T, N>) -> Transform<T, N>
  where
    T: One + Zero,
  {
    let mut m = Matrix::new(N + 1, N + 1);
    for i in 0..N {
      m[(i, i)] = vec[i].clone();
    }
    m[(N, N)] = T::one();
    Transform::new(m)
  }

  pub fn uniform_scale(v: T) -> Transform<T, N>
  where
    T: One + Zero,
  {
    let mut m = Matrix::new(N + 1, N + 1);
    for i in 0..N {
      m[(i, i)] = v.clone();
    }
    m[(N, N)] = T::one();
    Transform::new(m)
  }
}

impl<T, const N: usize> Mul for Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Transform<T, N>;
  fn mul(self, other: Transform<T, N>) -> Transform<T, N> {
    Transform::new(self.0 * other.0)
  }
}

impl<T, const N: usize> Mul<&Transform<T, N>> for &Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Transform<T, N>;
  fn mul(self, other: &Transform<T, N>) -> Transform<T, N> {
    Transform::new(&self.0 * &other.0)
  }
}

impl<T, const N: usize> Mul<&Point<T, N>> for &Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Point<T, N>;
  fn mul(self, other: &Point<T, N>) -> Point<T, N> {
    let v: Vector<T, N> = self * Vector::from(other.clone());
    v.into()
  }
}

// &t * &v = v
impl<T, const N: usize> Mul<&Vector<T, N>> for &Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Vector<T, N>;
  fn mul(self, other: &Vector<T, N>) -> Vector<T, N> {
    let mut v = Matrix::new(N + 1, 1);
    for i in 0..N {
      v[(i, 0)] = other.0[i].clone()
    }
    v[(N, 0)] = T::one();
    let ret = &self.0 * v;
    let normalizer = ret[(N, 0)].clone();
    Vector(array_init(|i| ret[(i, 0)].clone() / normalizer.clone()))
  }
}

// &t * v = v
impl<T, const N: usize> Mul<Vector<T, N>> for &Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Vector<T, N>;
  fn mul(self, other: Vector<T, N>) -> Vector<T, N> {
    self.mul(&other)
  }
}

impl<T, const N: usize> Mul<&Vector<T, N>> for Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Vector<T, N>;
  fn mul(self, other: &Vector<T, N>) -> Vector<T, N> {
    (&self).mul(other)
  }
}

impl<T, const N: usize> Mul<Point<T, N>> for Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Point<T, N>;
  fn mul(self, other: Point<T, N>) -> Point<T, N> {
    &self * &other
  }
}

impl<T, const N: usize> Mul<Point<T, N>> for &Transform<T, N>
where
  T: TransformScalar,
{
  type Output = Point<T, N>;
  fn mul(self, other: Point<T, N>) -> Point<T, N> {
    self * &other
  }
}

impl<T> Mul<&Polygon<T>> for &Transform<T, 2>
where
  T: TransformScalar,
{
  type Output = Polygon<T>;
  fn mul(self, other: &Polygon<T>) -> Polygon<T> {
    other.clone().map_points(|p| self * p)
  }
}

impl<T> Mul<Polygon<T>> for &Transform<T, 2>
where
  T: TransformScalar,
{
  type Output = Polygon<T>;
  fn mul(self, mut other: Polygon<T>) -> Polygon<T> {
    for pt in other.iter_mut() {
      *pt = self * pt.clone();
    }
    other
  }
}

impl<T> Mul<Polygon<T>> for Transform<T, 2>
where
  T: TransformScalar,
{
  type Output = Polygon<T>;
  fn mul(self, other: Polygon<T>) -> Polygon<T> {
    (&self).mul(other)
  }
}

impl<T> Mul<&Polygon<T>> for Transform<T, 2>
where
  T: TransformScalar,
{
  type Output = Polygon<T>;
  fn mul(self, other: &Polygon<T>) -> Polygon<T> {
    (&self).mul(other)
  }
}
