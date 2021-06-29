// This module contains strategies and shrinkers for:
//  * points
//  * polygons
// A Strategy is a way to generate a shrinkable value.
use crate::data::{Point, PointId, Polygon};
use crate::PolygonScalar;

use array_init::{array_init, try_array_init};
use core::ops::Range;
use num_bigint::BigInt;
use ordered_float::NotNan;
use proptest::prelude::*;
use proptest::strategy::*;
use proptest::test_runner::*;
use rand::SeedableRng;
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;

///////////////////////////////////////////////////////////////////////////////
// Shrinkable polygons

// Shrinking a polygon.
// 1. Cut off ears.
// 2. Simplify points.
pub struct ShrinkablePolygon<T: ValueTree> {
  points: Vec<ShrinkablePoint<T, 2>>,
  cut: Option<PointId>,
  uncut: Vec<PointId>,
  shrink: usize,
  done: bool,
}

impl<T: ValueTree> ShrinkablePolygon<T>
where
  <T as ValueTree>::Value: Clone + PolygonScalar,
{
  fn new(points: Vec<ShrinkablePoint<T, 2>>) -> Self {
    ShrinkablePolygon {
      points: points,
      cut: None,
      uncut: Vec::new(),
      shrink: 0,
      done: false,
    }
  }

  fn polygon(&self) -> Polygon<<T as ValueTree>::Value> {
    Polygon::new_unchecked(self.points.iter().map(|pt| pt.current()).collect())
  }
}

impl<T: ValueTree> ValueTree for ShrinkablePolygon<T>
where
  <T as ValueTree>::Value: Clone + PolygonScalar,
{
  type Value = Polygon<<T as ValueTree>::Value>;

  fn current(&self) -> Self::Value {
    if self.done {
      return self.polygon();
    }
    let mut poly = self.polygon();
    if let Some(cut) = self.cut {
      poly.rings[0].remove(cut.usize());
    }
    return poly;
  }

  // Reduce the complexity and return 'true' if something changed.
  // Return 'false' if the complexity cannot be reduced.
  fn simplify(&mut self) -> bool {
    if self.done {
      // eprintln!("Shrink done");
      return false;
    }

    // If we previously cut an ear and the tests are still failing, make the cut permanent.
    if let Some(cut) = self.cut {
      // eprintln!("Shrink cut {:?}", cut);
      self.points.remove(cut.usize());
      self.cut = None;
      self.uncut.clear();
    }

    // Look for more ears to cut.
    for pt in self.polygon().iter_boundary() {
      if !self.uncut.contains(&pt.point_id()) && pt.is_ear() {
        // eprintln!("Shrink ear: {:?}", pt.point_id());
        self.cut = Some(pt.point_id());
        return true;
      }
    }

    // No more ears can be cut. Let's try simplifying points:
    // eprintln!("Shrinking point: {}", self.shrink);
    while self.shrink < self.points.len() && !self.points[self.shrink].simplify() {
      self.shrink += 1;
      // eprintln!("Shrink next point: {}", self.shrink);
    }
    if self.shrink < self.points.len() {
      while self.polygon().validate().is_err() {
        // eprintln!("Bad point shrink. Undo: {}", self.shrink);
        if !self.points[self.shrink].complicate() {
          // eprintln!("Cannot undo. Abort");
          self.done = true;
          return true;
        }
      }
      return true;
    } else {
      self.done = true;
      return false;
    }
  }

  // The value has been shrunk so much that the test-cases no longer fail.
  // Undo the last shrink action and find another way to shrink the value.
  fn complicate(&mut self) -> bool {
    if self.done {
      return false;
    }

    if let Some(cut) = self.cut {
      // eprintln!("Undo cut");
      self.uncut.push(cut);
      self.cut = None;
      return true;
    }

    if self.shrink < self.points.len() {
      // eprintln!("Undo shrink");
      if !self.points[self.shrink].complicate() {
        self.shrink += 1;
      } else {
        while self.polygon().validate().is_err() {
          // eprintln!("Bad point unshrink. Undo: {}", self.shrink);
          if !self.points[self.shrink].complicate() {
            // eprintln!("Cannot undo. Abort");
            self.done = true;
            return true;
          }
        }
      }
      return true;
    } else {
      return false;
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Polygon strategy

#[derive(Debug, Clone)]
pub struct PolygonStrat<T>(T, Range<usize>);

impl<T> Strategy for PolygonStrat<T>
where
  T: Clone + std::fmt::Debug + Strategy,
  T::Value: Clone + PolygonScalar,
  T::Tree: Clone,
{
  type Tree = ShrinkablePolygon<T::Tree>;
  type Value = Polygon<T::Value>;
  fn new_tree(&self, runner: &mut TestRunner) -> Result<Self::Tree, Reason> {
    let n = runner.rng().gen_range(self.1.clone()).max(3);
    let mut points = Vec::with_capacity(n);
    let mut set = BTreeSet::new();
    let mut actual = Vec::new();
    for _ in 0..n {
      let pt = Point::new([self.0.clone(), self.0.clone()]).new_tree(runner)?;
      let current = pt.current();
      if set.insert(current.clone()) {
        points.push(pt);
        actual.push(current)
      }
    }
    // eprintln!("Generated points: {}/{}", points.len(), n);
    // eprintln!("Generating poly: {:?}", &actual);
    let rng = &mut rand::rngs::SmallRng::seed_from_u64(0);
    let poly = crate::algorithms::two_opt_moves(actual, rng).map_err(|err| err.to_string())?;

    assert_eq!(poly.rings[0].len(), points.len());
    // eprintln!("Re-ordering points");
    // FIXME: Super ugly:
    let mut new_points = Vec::new();
    for &pid in poly.rings[0].iter() {
      new_points.push(points[pid.usize()].clone());
    }

    Ok(ShrinkablePolygon::new(new_points))
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary polygons

impl<T: Arbitrary> Arbitrary for Polygon<T>
where
  T::Strategy: Clone,
  <<T as Arbitrary>::Strategy as Strategy>::Tree: Clone,
  T: PolygonScalar,
{
  type Strategy = PolygonStrat<T::Strategy>;
  type Parameters = (Range<usize>, T::Parameters);
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    let (size_range, t_params) = params;
    if size_range.is_empty() {
      PolygonStrat(T::arbitrary_with(t_params), 3..100)
    } else {
      PolygonStrat(T::arbitrary_with(t_params), size_range)
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Shrinkable point

#[derive(Clone)]
pub struct ShrinkablePoint<T, const N: usize> {
  point: Point<T, N>,
  shrink: usize,
  prev_shrink: Option<usize>,
}
impl<T, const N: usize> ValueTree for ShrinkablePoint<T, N>
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

///////////////////////////////////////////////////////////////////////////////
// Point strategy

impl<T, const N: usize> Strategy for Point<T, N>
where
  T: Clone + Debug + Strategy,
{
  type Tree = ShrinkablePoint<T::Tree, N>;
  type Value = Point<<T as Strategy>::Value, N>;
  fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
    let tree = ShrinkablePoint {
      point: Point {
        array: try_array_init(|i| self.array.index(i).new_tree(runner))?,
      },
      shrink: 0,
      prev_shrink: None,
    };

    Ok(tree)
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary point

impl<T: Arbitrary, const N: usize> Arbitrary for Point<T, N>
where
  T::Strategy: Clone,
  T::Parameters: Clone,
  T: Clone,
{
  type Strategy = Point<T::Strategy, N>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    Point {
      array: array_init(|_| T::arbitrary_with(params.clone())),
    }
  }
}

// FIXME: Move this impl to 'ordered_float' crate.
// impl Arbitrary for NotNan<f64> {
//   type Strategy = num::f64::Any;
//   type Parameters = ();
//   fn arbitrary_with(_params: ()) -> Self::Strategy {
//     POSITIVE | NEGATIVE | NORMAL | SUBNORMAL | ZERO;
//   }
// }

pub fn any_nn<const N: usize>() -> impl Strategy<Value = Point<NotNan<f64>, N>> {
  any::<Point<f64, N>>().prop_filter_map("Check for NaN", |pt| pt.try_into().ok())
}

pub fn any_r<const N: usize>() -> impl Strategy<Value = Point<BigInt, N>> {
  any::<Point<isize, N>>().prop_map(|pt| pt.cast(BigInt::from))
}

pub fn any_64<const N: usize>() -> impl Strategy<Value = Point<i64, N>> {
  any::<Point<i64, N>>()
}

pub fn any_8<const N: usize>() -> impl Strategy<Value = Point<i8, N>> {
  any::<Point<i8, N>>()
}
