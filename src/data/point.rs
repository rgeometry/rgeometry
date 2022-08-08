use array_init::{array_init, try_array_init};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::*;
use ordered_float::OrderedFloat;
use ordered_float::{FloatIsNan, NotNan};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::iter::Sum;
use std::ops::Deref;
use std::ops::Index;

use super::{Direction, Vector};
use crate::{Orientation, PolygonScalar, TotalOrd};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)] // Required for correctness!
pub struct Point<T, const N: usize = 2> {
  pub array: [T; N],
}

impl<T: TotalOrd, const N: usize> TotalOrd for Point<T, N> {
  fn total_cmp(&self, other: &Self) -> Ordering {
    for i in 0..N {
      match self.array[i].total_cmp(&other.array[i]) {
        Ordering::Equal => continue,
        other => return other,
      }
    }
    Ordering::Equal
  }
}

impl<T: TotalOrd, const N: usize> PartialEq for Point<T, N> {
  fn eq(&self, other: &Self) -> bool {
    self.total_cmp(other).is_eq()
  }
}
impl<T: TotalOrd, const N: usize> Eq for Point<T, N> {}

impl<T: TotalOrd, const N: usize> PartialOrd for Point<T, N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.total_cmp(other))
  }
}

impl<T: TotalOrd, const N: usize> Ord for Point<T, N> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.total_cmp(other)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct PointSoS<'a, T, const N: usize = 2> {
  pub index: u32,
  pub point: &'a Point<T, N>,
}

impl<'a, T: TotalOrd, const N: usize> TotalOrd for PointSoS<'a, T, N> {
  fn total_cmp(&self, other: &Self) -> Ordering {
    (self.index, self.point).total_cmp(&(other.index, other.point))
  }
}

impl<'a, T: TotalOrd, const N: usize> PartialEq for PointSoS<'a, T, N> {
  fn eq(&self, other: &Self) -> bool {
    self.total_cmp(other).is_eq()
  }
}
impl<'a, T: TotalOrd, const N: usize> Eq for PointSoS<'a, T, N> {}

impl<'a, T: TotalOrd, const N: usize> PartialOrd for PointSoS<'a, T, N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.total_cmp(other))
  }
}

impl<'a, T: TotalOrd, const N: usize> Ord for PointSoS<'a, T, N> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.total_cmp(other)
  }
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
  // See `cmp_distance_to` for a safe way to compare distances in 2D.
  pub fn squared_euclidean_distance<F>(&self, rhs: &Point<T, N>) -> F
  where
    T: Clone + Into<F>,
    F: NumOps + Clone + Sum,
  {
    self
      .array
      .iter()
      .zip(rhs.array.iter())
      .map(|(a, b)| {
        let diff = a.clone().into() - b.clone().into();
        diff.clone() * diff
      })
      .sum()
  }

  // Similar to num_traits::identities::Zero but doesn't require an Add impl.
  pub fn zero() -> Self
  where
    T: Sum,
  {
    Point {
      array: array_init(|_| std::iter::empty().sum()),
    }
  }

  pub fn map<U, F>(&self, f: F) -> Point<U, N>
  where
    T: Clone,
    F: Fn(T) -> U,
  {
    Point {
      array: array_init(|i| f(self.array[i].clone())),
    }
  }

  pub fn cast<U>(&self) -> Point<U, N>
  where
    T: Clone + Into<U>,
  {
    Point {
      array: array_init(|i| self.array[i].clone().into()),
    }
  }

  pub fn to_float(&self) -> Point<OrderedFloat<f64>, N>
  where
    T: Clone + Into<f64>,
  {
    self.map(|v| OrderedFloat(v.into()))
  }
}

impl<T, const N: usize> Index<usize> for Point<T, N> {
  type Output = T;
  fn index(&self, key: usize) -> &T {
    self.array.index(key)
  }
}

impl<const N: usize> TryFrom<Point<f64, N>> for Point<NotNan<f64>, N> {
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

// Methods on two-dimensional points.
impl<T: PolygonScalar> Point<T> {
  pub fn cmp_distance_to(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering {
    T::cmp_dist(self, p, q)
  }

  /// Determine the direction you have to turn if you walk from `p1`
  /// to `p2` to `p3`.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use rgeometry::data::Point;
  /// # use rgeometry::Orientation;
  /// let p1 = Point::new([ 0, 0 ]);
  /// let p2 = Point::new([ 0, 1 ]); // One unit above p1.
  /// // (0,0) -> (0,1) -> (0,2) == Orientation::CoLinear
  /// assert!(Point::orient(&p1, &p2, &Point::new([ 0, 2 ])).is_colinear());
  /// // (0,0) -> (0,1) -> (-1,2) == Orientation::CounterClockWise
  /// assert!(Point::orient(&p1, &p2, &Point::new([ -1, 2 ])).is_ccw());
  /// // (0,0) -> (0,1) -> (1,2) == Orientation::ClockWise
  /// assert!(Point::orient(&p1, &p2, &Point::new([ 1, 2 ])).is_cw());
  /// ```
  ///
  pub fn orient(p1: &Point<T, 2>, p2: &Point<T, 2>, p3: &Point<T, 2>) -> Orientation {
    Orientation::new(p1, p2, p3)
  }

  pub fn orient_along_direction(
    p1: &Point<T, 2>,
    direction: Direction<'_, T, 2>,
    p2: &Point<T, 2>,
  ) -> Orientation {
    match direction {
      Direction::Vector(v) => Orientation::along_vector(p1, v, p2),
      Direction::Through(p) => Orientation::new(p1, p, p2),
    }
  }

  pub fn orient_along_vector(
    p1: &Point<T, 2>,
    vector: &Vector<T, 2>,
    p2: &Point<T, 2>,
  ) -> Orientation {
    Orientation::along_vector(p1, vector, p2)
  }

  pub fn orient_along_perp_vector(
    p1: &Point<T, 2>,
    vector: &Vector<T, 2>,
    p2: &Point<T, 2>,
  ) -> Orientation {
    Orientation::along_perp_vector(p1, vector, p2)
  }

  pub fn all_colinear(pts: &[Point<T>]) -> bool {
    if pts.len() < 3 {
      return true;
    }
    pts
      .iter()
      .all(|pt| Point::orient(&pts[0], &pts[1], pt).is_colinear())
  }

  /// Docs?
  pub fn ccw_cmp_around(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering {
    self.ccw_cmp_around_with(&Vector([T::from_constant(1), T::from_constant(0)]), p, q)
  }

  pub fn ccw_cmp_around_with(
    &self,
    z: &Vector<T, 2>,
    p: &Point<T, 2>,
    q: &Point<T, 2>,
  ) -> Ordering {
    Orientation::ccw_cmp_around_with(z, self, p, q)
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

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod tests {
  use super::*;
  use crate::testing::*;
  use crate::Orientation::*;

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn cmp_dist_i8_fuzz(pt1: Point<i8, 2>, pt2: Point<i8, 2>, pt3: Point<i8, 2>) {
    let pt1_big: Point<BigInt, 2> = pt1.cast();
    let pt2_big: Point<BigInt, 2> = pt2.cast();
    let pt3_big: Point<BigInt, 2> = pt3.cast();
    prop_assert_eq!(
      pt1.cmp_distance_to(&pt2, &pt3),
      pt1_big.cmp_distance_to(&pt2_big, &pt3_big)
    );
  }

  #[proptest]
  fn squared_euclidean_distance_fuzz(
    #[strategy(any_nn::<2>())] pt1: Point<NotNan<f64>, 2>,
    #[strategy(any_nn::<2>())] pt2: Point<NotNan<f64>, 2>,
  ) {
    let _out: f64 = pt1.squared_euclidean_distance(&pt2);
  }

  #[proptest]
  fn squared_euclidean_distance_i8_fuzz(pt1: Point<i8, 2>, pt2: Point<i8, 2>) {
    let _out: i64 = pt1.squared_euclidean_distance(&pt2);
  }

  #[proptest]
  fn cmp_around_fuzz_nn(
    #[strategy(any_nn::<2>())] pt1: Point<NotNan<f64>, 2>,
    #[strategy(any_nn::<2>())] pt2: Point<NotNan<f64>, 2>,
    #[strategy(any_nn::<2>())] pt3: Point<NotNan<f64>, 2>,
  ) {
    let _ = pt1.ccw_cmp_around(&pt2, &pt3);
  }

  #[proptest]
  fn cmp_around_fuzz_i8(pt1: Point<i8, 2>, pt2: Point<i8, 2>, pt3: Point<i8, 2>) {
    let _ = pt1.ccw_cmp_around(&pt2, &pt3);
  }

  #[proptest]
  fn bigint_colinear(
    #[strategy(any_r::<2>())] pt1: Point<BigInt, 2>,
    #[strategy(any_r::<2>())] pt2: Point<BigInt, 2>,
  ) {
    let diff = &pt2 - &pt1;
    let pt3 = &pt2 + &diff;
    prop_assert!(Point::orient(&pt1, &pt2, &pt3).is_colinear())
  }

  #[proptest]
  fn bigint_not_colinear(
    #[strategy(any_r::<2>())] pt1: Point<BigInt, 2>,
    #[strategy(any_r::<2>())] pt2: Point<BigInt, 2>,
  ) {
    let diff = &pt2 - &pt1;
    let pt3 = &pt2 + &diff + &Vector([BigInt::from(1), BigInt::from(1)]);
    prop_assert!(!Point::orient(&pt1, &pt2, &pt3).is_colinear())
  }

  // i8 and BigInt uses different algorithms for computing orientation but the results
  // should be identical.
  #[proptest]
  fn orient_bigint_i8_prop(pt1: Point<i8, 2>, pt2: Point<i8, 2>, pt3: Point<i8, 2>) {
    let pt1_big: Point<BigInt, 2> = pt1.cast();
    let pt2_big: Point<BigInt, 2> = pt2.cast();
    let pt3_big: Point<BigInt, 2> = pt3.cast();

    prop_assert_eq!(
      Point::orient(&pt1, &pt2, &pt3),
      Point::orient(&pt1_big, &pt2_big, &pt3_big)
    )
  }

  // i8 and f64 uses different algorithms for computing orientation but the results
  // should be identical.
  #[proptest]
  fn orient_f64_i8_prop(pt1: Point<i8, 2>, pt2: Point<i8, 2>, pt3: Point<i8, 2>) {
    let pt1_big: Point<OrderedFloat<f64>, 2> = pt1.to_float();
    let pt2_big: Point<OrderedFloat<f64>, 2> = pt2.to_float();
    let pt3_big: Point<OrderedFloat<f64>, 2> = pt3.to_float();

    prop_assert_eq!(
      Point::orient(&pt1, &pt2, &pt3),
      Point::orient(&pt1_big, &pt2_big, &pt3_big)
    )
  }

  #[proptest]
  fn orientation_reverse(pt1: Point<i64, 2>, pt2: Point<i64, 2>, pt3: Point<i64, 2>) {
    let abc = Point::orient(&pt1, &pt2, &pt3);
    let cba = Point::orient(&pt3, &pt2, &pt1);
    prop_assert_eq!(abc, cba.reverse())
  }

  #[test]
  fn test_turns() {
    assert_eq!(
      Point::orient(
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
      Point::orient(
        &Point::new([0, 0]),
        &Point::new([0, 1]),
        &Point::new([2, 2])
      ),
      ClockWise
    );
    assert_eq!(
      Point::orient(
        &Point::new_nn([0.0, 0.0]),
        &Point::new_nn([0.0, 1.0]),
        &Point::new_nn([2.0, 2.0])
      ),
      ClockWise
    );

    assert_eq!(
      Point::orient(
        &Point::new([0, 0]),
        &Point::new([0, 1]),
        &Point::new([-2, 2])
      ),
      CounterClockWise
    );
    assert_eq!(
      Point::orient(
        &Point::new([0, 0]),
        &Point::new([0, 0]),
        &Point::new([0, 0])
      ),
      CoLinear
    );
  }

  #[test]
  fn unit_1() {
    assert_eq!(
      Point::orient(
        &Point::new([0, 0]),
        &Point::new([1, 0]),
        &Point::new([1, 0])
      ),
      CoLinear
    );
    assert_eq!(
      Point::orient(
        &Point::new([0, 0]),
        &Point::new([1, 0]),
        &Point::new([2, 0])
      ),
      CoLinear
    );
    assert_eq!(
      Point::orient(
        &Point::new([1, 0]),
        &Point::new([2, 0]),
        &Point::new([0, 0])
      ),
      CoLinear
    );
    assert_eq!(
      Point::orient(
        &Point::new([1, 0]),
        &Point::new([2, 0]),
        &Point::new([1, 0])
      ),
      CoLinear
    );
  }

  #[test]
  fn unit_2() {
    assert_eq!(
      Point::orient(
        &Point::new([1, 0]),
        &Point::new([0, 6]),
        &Point::new([0, 8])
      ),
      ClockWise
    );
  }

  #[test]
  fn unit_3() {
    assert_eq!(
      Point::orient(
        &Point::new([-12_i8, -126]),
        &Point::new([-12, -126]),
        &Point::new([0, -126])
      ),
      CoLinear
    );
  }
}
