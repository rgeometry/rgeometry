use num_traits::Zero;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Mul;

pub trait MatrixMul: Clone + Zero + AddAssign + Mul<Self, Output = Self> {}

impl<T> MatrixMul for T where T: Clone + Zero + AddAssign + Mul<Self, Output = Self> {}

#[derive(Clone, Debug)]
pub struct Matrix<T> {
  nrows: usize,
  ncols: usize,
  elements: Vec<T>,
}

impl<T> Matrix<T> {
  pub fn new(nrows: usize, ncols: usize) -> Matrix<T>
  where
    T: Zero,
  {
    let mut vec = Vec::with_capacity(nrows * ncols);
    for _i in 0..nrows * ncols {
      vec.push(T::zero())
    }
    Matrix {
      nrows,
      ncols,
      elements: vec,
    }
  }
  fn validate(&self) {
    assert_eq!(self.elements.len(), self.nrows * self.ncols)
  }
  pub fn nrows(&self) -> usize {
    self.nrows
  }
  pub fn ncols(&self) -> usize {
    self.ncols
  }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
  type Output = T;
  fn index(&self, key: (usize, usize)) -> &T {
    self.elements.index(key.0 + key.1 * self.ncols)
  }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
  fn index_mut(&mut self, key: (usize, usize)) -> &mut T {
    self.elements.index_mut(key.0 + key.1 * self.ncols)
  }
}

impl<T> Mul<&Matrix<T>> for &Matrix<T>
where
  T: Clone + Zero + AddAssign + Mul<T, Output = T>,
{
  type Output = Matrix<T>;
  // n*m * m*p = n*p
  fn mul(self, other: &Matrix<T>) -> Matrix<T> {
    let n = self.nrows;
    let m = self.ncols;
    let p = other.ncols;
    assert_eq!(self.ncols, other.nrows);
    let mut out = Matrix::new(n, p);
    for i in 0..n {
      for j in 0..p {
        for k in 0..m {
          out[(i, j)] += self[(i, k)].clone() * other[(k, j)].clone()
        }
      }
    }
    out.validate();
    out
  }
}

impl<T> Mul<Matrix<T>> for &Matrix<T>
where
  T: Clone + Zero + AddAssign + Mul<T, Output = T>,
{
  type Output = Matrix<T>;
  fn mul(self, other: Matrix<T>) -> Matrix<T> {
    self * &other
  }
}
impl<T> Mul<&Matrix<T>> for Matrix<T>
where
  T: Clone + Zero + AddAssign + Mul<T, Output = T>,
{
  type Output = Matrix<T>;
  fn mul(self, other: &Matrix<T>) -> Matrix<T> {
    &self * other
  }
}
impl<T> Mul<Matrix<T>> for Matrix<T>
where
  T: Clone + Zero + AddAssign + Mul<T, Output = T>,
{
  type Output = Matrix<T>;
  fn mul(self, other: Matrix<T>) -> Matrix<T> {
    &self * &other
  }
}
