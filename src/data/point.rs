use array_init::{array_init, try_array_init};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::*;
use ordered_float::{FloatIsNan, NotNan};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::Index;
use std::ops::Neg;

use super::Vector;
use crate::array::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)] // Required for correctness!
pub struct Point<T, const N: usize> {
  pub array: [T; N],
}

// Random sampling.
impl<T, const N: usize> Distribution<Point<T, N>> for Standard
where
  Standard: Distribution<T>,
{
  // FIXME: Unify with code for Vector.
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point<T, N> {
    Point {
      array: array_init(|_| rng.gen()),
    }
  }
}

// Methods on N-dimensional points.
impl<T, const N: usize> Point<T, N> {
  pub const fn new(array: [T; N]) -> Point<T, N> {
    Point { array }
  }

  /// # Panics
  ///
  /// Panics if any of the inputs are NaN.
  pub fn new_nn(array: [T; N]) -> Point<NotNan<T>, N>
  where
    T: Float,
  {
    Point::new(array_init(|i| NotNan::new(array[i]).unwrap()))
  }

  pub fn as_vec(&self) -> &Vector<T, N> {
    self.into()
  }

  // Warning: May cause arithmetic overflow.
  pub fn cmp_distance_to(&self, p: &Point<T, N>, q: &Point<T, N>) -> Ordering
  where
    T: Clone + Zero + Ord + NumOps + crate::Extended,
  {
    self
      .squared_euclidean_distance(p)
      .cmp(&self.squared_euclidean_distance(q))
  }

  // Warning: May cause arithmetic overflow.
  pub fn squared_euclidean_distance(&self, rhs: &Point<T, N>) -> T::ExtendedSigned
  where
    T: Clone + Zero + NumOps + crate::Extended,
  {
    self
      .array
      .iter()
      .zip(rhs.array.iter())
      .map(|(a, b)| {
        let diff = a.clone().extend_signed() - b.clone().extend_signed();
        diff.clone() * diff
      })
      .sum()
  }

  // Similar to num_traits::identities::Zero but doesn't require an Add impl.
  pub fn zero() -> Self
  where
    T: Zero,
  {
    Point {
      array: array_init(|_| Zero::zero()),
    }
  }

  pub fn cast<U, F>(&self, f: F) -> Point<U, N>
  where
    T: Clone,
    F: Fn(T) -> U,
  {
    Point {
      array: array_init(|i| f(self.array[i].clone())),
    }
  }
}

impl<T, const N: usize> Index<usize> for Point<T, N> {
  type Output = T;
  fn index(&self, key: usize) -> &T {
    self.array.index(key)
  }
}

impl<'a, const N: usize> TryFrom<Point<f64, N>> for Point<NotNan<f64>, N> {
  type Error = FloatIsNan;
  fn try_from(point: Point<f64, N>) -> Result<Point<NotNan<f64>, N>, FloatIsNan> {
    Ok(Point {
      array: try_array_init(|i| NotNan::try_from(point.array[i]))?,
    })
  }
}

impl<T> From<(T, T)> for Point<T, 2> {
  fn from(point: (T, T)) -> Point<T, 2> {
    Point {
      array: [point.0, point.1],
    }
  }
}

impl From<Point<i64, 2>> for Point<BigInt, 2> {
  fn from(point: Point<i64, 2>) -> Point<BigInt, 2> {
    Point {
      array: [point.array[0].into(), point.array[1].into()],
    }
  }
}

impl<'a, const N: usize> From<&'a Point<BigRational, N>> for Point<f64, N> {
  fn from(point: &Point<BigRational, N>) -> Point<f64, N> {
    Point {
      array: array_init(|i| point.array[i].to_f64().unwrap()),
    }
  }
}

impl<const N: usize> From<Point<BigRational, N>> for Point<f64, N> {
  fn from(point: Point<BigRational, N>) -> Point<f64, N> {
    Point {
      array: array_init(|i| point.array[i].to_f64().unwrap()),
    }
  }
}

impl<'a, const N: usize> From<&'a Point<f64, N>> for Point<BigRational, N> {
  fn from(point: &Point<f64, N>) -> Point<BigRational, N> {
    Point {
      array: array_init(|i| BigRational::from_f64(point.array[i]).unwrap()),
    }
  }
}

impl<const N: usize> From<Point<f64, N>> for Point<BigRational, N> {
  fn from(point: Point<f64, N>) -> Point<BigRational, N> {
    Point {
      array: array_init(|i| BigRational::from_f64(point.array[i]).unwrap()),
    }
  }
}

impl<T, const N: usize> From<Vector<T, N>> for Point<T, N> {
  fn from(vector: Vector<T, N>) -> Point<T, N> {
    Point { array: vector.0 }
  }
}

// impl<T, const N: usize> AsRef<Vector<T, N>> for Point<T, N> {
//   fn as_ref(&self) -> &Vector<T, N> {
//     self.into()
//   }
// }

// pub fn orientation<T>(p: &Point<T, 2>, q: &Point<T, 2>, r: &Point<T, 2>) -> Orientation
// where
//   T: Sub<T, Output = T> + Clone + Mul<T, Output = T> + Ord,
//   // for<'a> &'a T: Sub<Output = T>,
// {
//   raw_arr_turn(&p.array, &q.array, &r.array)
// }

// Methods on two-dimensional points.
impl<T> Point<T, 2> {
  pub fn orientation(&self, q: &Point<T, 2>, r: &Point<T, 2>) -> Orientation
  where
    T: Clone + NumOps + Ord + crate::Extended,
  {
    Orientation::new(&self.array, &q.array, &r.array)
  }

  /// Docs?
  pub fn ccw_cmp_around(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Zero + One + Neg<Output = T> + crate::Extended + Signed,
    // for<'a> &'a T: Mul<&'a T, Output = T>,
  {
    self.ccw_cmp_around_with(&Vector([T::one(), T::zero()]), p, q)
  }

  pub fn ccw_cmp_around_with(&self, z: &Vector<T, 2>, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Neg<Output = T> + crate::Extended + Signed,
    // for<'a> &'a T: Mul<Output = T>,
  {
    ccw_cmp_around_with(&z.0, &self, &p, &q)
  }
}

// FIXME: Use a macro
impl<T> Point<T, 1> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
}
impl<T> Point<T, 2> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
  pub fn y_coord(&self) -> &T {
    &self.array[1]
  }
}
impl<T> Point<T, 3> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
  pub fn y_coord(&self) -> &T {
    &self.array[1]
  }
  pub fn z_coord(&self) -> &T {
    &self.array[2]
  }
}

impl<T, const N: usize> Deref for Point<T, N> {
  type Target = [T; N];
  fn deref(&self) -> &[T; N] {
    &self.array
  }
}

mod add;
mod sub;

// // Sigh, should relax Copy to Clone.
// impl<T, const N: usize> Zero for Point<T, N>
// where
//   T: Zero + Copy + Add + Add<Output = T>,
// {
//   fn zero() -> Point<T, N> {
//     Point([Zero::zero(); N])
//   }
//   fn is_zero(&self) -> bool {
//     self.0.iter().all(Zero::is_zero)
//   }
// }

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::testing::*;
  use crate::Orientation::*;

  use proptest::prelude::*;

  proptest! {
    #[test]
    fn squared_euclidean_distance_fuzz(pt1 in any_nn::<2>(), pt2 in any_nn::<2>()) {
      let _ = pt1.squared_euclidean_distance(&pt2);
    }

    #[test]
    fn cmp_around_fuzz_nn(pt1 in any_nn(), pt2 in any_nn(), pt3 in any_nn()) {
      let _ = pt1.ccw_cmp_around(&pt2, &pt3);
    }

    #[test]
    fn cmp_around_fuzz_i8(pt1 in any_8(), pt2 in any_8(), pt3 in any_8()) {
      let _ = pt1.ccw_cmp_around(&pt2, &pt3);
    }

    #[test]
    fn bigint_colinear(pt1 in any_r(), pt2 in any_r()) {
      let diff = &pt2 - &pt1;
      let pt3 = &pt2 + &diff;
      prop_assert!(Orientation::is_colinear(&pt1, &pt2, &pt3))
    }

    #[test]
    fn bigint_not_colinear(pt1 in any_r(), pt2 in any_r()) {
      let diff = &pt2 - &pt1;
      let pt3 = &pt2 + &diff + &Vector([BigInt::from(1),BigInt::from(1)]);
      prop_assert!(!Orientation::is_colinear(&pt1, &pt2, &pt3))
    }

    #[test]
    fn orientation_reverse(pt1 in any_64(), pt2 in any_64(), pt3 in any_64()) {
      let abc = Orientation::new(&pt1, &pt2, &pt3);
      let cba = Orientation::new(&pt3, &pt2, &pt1);
      prop_assert_eq!(abc, cba.reverse())
    }
  }

  #[test]
  fn test_turns() {
    assert_eq!(
      Orientation::new(
        &Point::new([0, 0]),
        &Point::new([1, 1]),
        &Point::new([2, 2])
      ),
      CoLinear
    );
    assert_eq!(
      Orientation::new(
        &Point::new_nn([0.0, 0.0]),
        &Point::new_nn([1.0, 1.0]),
        &Point::new_nn([2.0, 2.0])
      ),
      CoLinear
    );

    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([2, 2])),
      ClockWise
    );
    assert_eq!(
      Point::new_nn([0.0, 0.0]).orientation(&Point::new_nn([0.0, 1.0]), &Point::new_nn([2.0, 2.0])),
      ClockWise
    );

    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([-2, 2])),
      CounterClockWise
    );
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 0]), &Point::new([0, 0])),
      CoLinear
    );
  }

  #[test]
  fn unit_1() {
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([1, 0]), &Point::new([1, 0])),
      CoLinear
    );
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([1, 0]), &Point::new([2, 0])),
      CoLinear
    );
    assert_eq!(
      Point::new([1, 0]).orientation(&Point::new([2, 0]), &Point::new([0, 0])),
      CoLinear
    );
    assert_eq!(
      Point::new([1, 0]).orientation(&Point::new([2, 0]), &Point::new([1, 0])),
      CoLinear
    );
  }

  #[test]
  fn unit_2() {
    assert_eq!(
      Point::new([1, 0]).orientation(&Point::new([0, 6]), &Point::new([0, 8])),
      ClockWise
    );
  }

  #[test]
  fn unit_3() {
    assert_eq!(
      Point::new([-12_i8, -126]).orientation(&Point::new([-12, -126]), &Point::new([0, -126])),
      CoLinear
    );
  }
}
