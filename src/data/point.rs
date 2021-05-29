use array_init::{array_init, try_array_init};
use num_rational::BigRational;
use num_traits::*;
use ordered_float::{FloatIsNan, NotNan, OrderedFloat};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Debug;
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
impl<T: Clone, const N: usize> Point<T, N> {
  pub fn new(array: [T; N]) -> Point<T, N> {
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

  pub fn cmp_distance_to(&self, p: &Point<T, N>, q: &Point<T, N>) -> Ordering
  where
    T: Clone + Zero + Ord + NumOps,
    // for<'a> &'a T: Mul<&'a T, Output = T> + Sub<&'a T, Output = T>,
  {
    self
      .squared_euclidean_distance(p)
      .cmp(&self.squared_euclidean_distance(q))
  }

  pub fn squared_euclidean_distance(&self, rhs: &Point<T, N>) -> T
  where
    T: Clone + Zero + NumOps,
    // for<'a> &'a T: Mul<&'a T, Output = T> + Sub<&'a T, Output = T>,
  {
    self
      .array
      .iter()
      .zip(rhs.array.iter())
      .fold(T::zero(), |sum, (a, b)| {
        let diff: T = a.clone() - b.clone();
        sum + diff.clone() * diff
      })
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

impl<'a, const N: usize> TryFrom<Point<f64, N>> for Point<BigRational, N> {
  type Error = ();
  fn try_from(point: Point<f64, N>) -> Result<Point<BigRational, N>, ()> {
    Ok(Point {
      array: try_array_init(|i| BigRational::from_float(point.array[i]).ok_or(()))?,
    })
  }
}

impl<'a, const N: usize> From<&'a Point<BigRational, N>> for Point<f64, N> {
  fn from(point: &Point<BigRational, N>) -> Point<f64, N> {
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
    T: Clone + NumOps + Ord,
    // for<'a> &'a T: Sub<Output = T>,
  {
    raw_arr_turn(&self.array, &q.array, &r.array)
  }

  /// Docs?
  pub fn ccw_cmp_around(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Zero + One + Neg<Output = T>,
    // for<'a> &'a T: Mul<&'a T, Output = T>,
  {
    self.ccw_cmp_around_with(&Vector([T::one(), T::zero()]), p, q)
  }

  pub fn ccw_cmp_around_with(&self, z: &Vector<T, 2>, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Neg<Output = T>,
    // for<'a> &'a T: Mul<Output = T>,
  {
    ccw_cmp_around_origin_with(&z.0, &(p - self).0, &(q - self).0)
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
mod tests {
  use super::*;
  use crate::Orientation::*;

  use ordered_float::Float;
  use ordered_float::NotNan;
  use std::convert::TryInto;
  use std::fmt::Debug;
  use std::ops::IndexMut;

  use proptest::array::*;
  use proptest::collection::*;
  use proptest::num;
  use proptest::prelude::*;
  use proptest::strategy::*;
  use proptest::test_runner::*;

  pub struct PointValueTree<T, const N: usize> {
    point: Point<T, N>,
    shrink: usize,
    prev_shrink: Option<usize>,
  }
  impl<T, const N: usize> ValueTree for PointValueTree<T, N>
  where
    T: ValueTree,
  {
    type Value = Point<<T as ValueTree>::Value, N>;
    fn current(&self) -> Point<T::Value, N> {
      Point {
        array: array_init(|i| self.point.array.index(i).current()),
      }
    }
    fn simplify(&mut self) -> bool {
      for ix in self.shrink..N {
        if !self.point.array.index_mut(ix).simplify() {
          self.shrink = ix + 1;
        } else {
          self.prev_shrink = Some(ix);
          return true;
        }
      }
      false
    }
    fn complicate(&mut self) -> bool {
      match self.prev_shrink {
        None => false,
        Some(ix) => {
          if self.point.array.index_mut(ix).complicate() {
            true
          } else {
            self.prev_shrink = None;
            false
          }
        }
      }
    }
  }

  impl<T, const N: usize> Strategy for Point<T, N>
  where
    T: Clone + Debug + Strategy,
  {
    type Tree = PointValueTree<T::Tree, N>;
    type Value = Point<<T as Strategy>::Value, N>;
    fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
      let tree = PointValueTree {
        point: Point {
          array: try_array_init(|i| self.array.index(i).new_tree(runner))?,
        },
        shrink: 0,
        prev_shrink: None,
      };

      Ok(tree)
    }
  }

  impl<const N: usize> Arbitrary for Point<f64, N> {
    type Strategy = Point<num::f64::Any, N>;
    type Parameters = ();
    fn arbitrary_with(_params: ()) -> Self::Strategy {
      use num::f64::*;
      let strat = POSITIVE | NEGATIVE | NORMAL | SUBNORMAL | ZERO;
      Point {
        array: array_init(|_| strat),
      }
    }
  }

  pub fn any_nn<const N: usize>() -> impl Strategy<Value = Point<NotNan<f64>, N>> {
    any::<Point<f64, N>>().prop_filter_map("Check for NaN", |pt| pt.try_into().ok())
  }

  proptest! {
    #[test]
    fn squared_euclidean_distance_fuzz(pt1: Point<f64,2>, pt2: Point<f64,2>) {
      let _ = pt1.squared_euclidean_distance(&pt2);
    }

    #[test]
    fn cmp_around_fuzz(pt1 in any_nn(), pt2 in any_nn(), pt3 in any_nn()) {
      let _ = pt1.ccw_cmp_around(&pt2, &pt3);
    }
  }

  #[test]
  fn test_turns() {
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([1, 1]), &Point::new([2, 2])),
      CoLinear
    );
    assert_eq!(
      Point::new_nn([0.0, 0.0]).orientation(&Point::new_nn([1.0, 1.0]), &Point::new_nn([2.0, 2.0])),
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
}
