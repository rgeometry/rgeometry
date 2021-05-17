use array_init::array_init;
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
      nrows: nrows,
      ncols: ncols,
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

impl<'a, 'b, T> Mul<&'b Matrix<T>> for &'a Matrix<T>
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

impl<'a, T> Mul<Matrix<T>> for &'a Matrix<T>
where
  T: Clone + Zero + AddAssign + Mul<T, Output = T>,
{
  type Output = Matrix<T>;
  fn mul(self, other: Matrix<T>) -> Matrix<T> {
    self * &other
  }
}
impl<'a, T> Mul<&'a Matrix<T>> for Matrix<T>
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

// pub struct MatrixPlus<T, const N: usize>(ArrayPlus<ArrayPlus<T, N>, N>);

// #[derive(Clone)]
// // Isomorphic to [T; N+1]
// pub struct ArrayPlus<T, const N: usize> {
//   head: T,
//   tail: [T; N],
// }

// impl<T, const N: usize> ArrayPlus<T, N>
// where
//   for<'a> T: Clone + AddAssign<T>,
// {
//   pub fn sum(&self) -> T {
//     let mut acc = self.head.clone();
//     for elt in self.tail.iter() {
//       acc += elt.clone()
//     }
//     acc
//   }
// }

// pub fn mult_mv<T, const N: usize>(m: MatrixPlus<T, N>, v: ArrayPlus<T, N>) -> ArrayPlus<T, N>
// where
//   T: Mul<T, Output = T> + AddAssign + Clone,
// {
//   let head = &m.0.head * &v;
//   let tail = array_init(|i| (m.0.tail.index(i) * &v).sum());
//   ArrayPlus {
//     head: head.sum(),
//     tail: tail,
//   }
// }

// fn mult_vm<T, const N: usize>(v: ArrayPlus<T, N>, m: MatrixPlus<T, N>) -> MatrixPlus<T, N>
// where
//   T: Mul<T, Output = T> + AddAssign + Clone,
// {
//   let head = m.0.head.clone() * v.head.clone();
//   // let x = m.0.tail[0];
//   let tail: [ArrayPlus<T, N>; N] = array_init(|i| (&m).0.tail[i].clone() * v.tail[i].clone());
//   MatrixPlus(ArrayPlus {
//     head: head,
//     tail: tail,
//   })
// }

// impl<T, const N: usize> Mul for ArrayPlus<T, N>
// where
//   for<'a> &'a T: Mul<&'a T, Output = T>,
// {
//   type Output = ArrayPlus<T, N>;
//   fn mul(self, other: ArrayPlus<T, N>) -> ArrayPlus<T, N> {
//     ArrayPlus {
//       head: &self.head * &other.head,
//       tail: array_init(|i| self.tail.index(i) * other.tail.index(i)),
//     }
//   }
// }

// impl<'a, T, const N: usize> Mul<&'a ArrayPlus<T, N>> for &'a ArrayPlus<T, N>
// where
//   T: Mul<T, Output = T> + Clone,
// {
//   type Output = ArrayPlus<T, N>;
//   fn mul(self, other: &ArrayPlus<T, N>) -> ArrayPlus<T, N> {
//     ArrayPlus {
//       head: self.head.clone() * other.head.clone(),
//       tail: array_init(|i| self.tail.index(i).clone() * other.tail.index(i).clone()),
//     }
//   }
// }

// impl<T, const N: usize> Mul<T> for ArrayPlus<T, N>
// where
//   T: Mul<T, Output = T> + Clone,
// {
//   type Output = ArrayPlus<T, N>;
//   fn mul(self, other: T) -> ArrayPlus<T, N> {
//     ArrayPlus {
//       head: self.head.clone() * other.clone(),
//       tail: array_init(|i| self.tail.index(i).clone() * other.clone()),
//     }
//   }
// }

// // matrix * vector = vector
// pub fn mult_mv<T>(m: Vec<Vec<T>>, v: &Vec<T>) -> Vec<T>
// where
//   for<'a> &'a T: Mul<&'a T, Output = T>,
//   T: std::iter::Sum,
// {
//   m.iter()
//     .map(|r| r.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
//     .collect()
// }
