// This module contains strategies and shrinkers for:
//  * points
//  * polygons
// A Strategy is a way to generate a shrinkable value.
use crate::data::{
  Direction, Direction_, Line, LineSoS, LineSoS_, Line_, Point, PointId, Polygon, PolygonConvex,
  Triangle, Vector,
};
use crate::PolygonScalar;

use array_init::{array_init, try_array_init};
use core::ops::Range;
use num::BigRational;
use num_bigint::BigInt;
use num_traits::*;
use ordered_float::NotNan;
use proptest::arbitrary::*;
use proptest::collection::*;
use proptest::prelude::*;
use proptest::strategy::*;
use proptest::test_runner::*;
use rand::distributions::uniform::SampleUniform;
use rand::SeedableRng;
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;

type Mapped<I, O> = Map<StrategyFor<I>, fn(_: I) -> O>;
type FilterMapped<I, O> = FilterMap<StrategyFor<I>, fn(_: I) -> Option<O>>;

///////////////////////////////////////////////////////////////////////////////
// Shrinkable polygons

// Shrinking a polygon.
// 1. Cut off ears.
// 2. Simplify points.
pub struct ShrinkablePolygon<T: ValueTree> {
  points: Vec<ShrinkablePoint<T, 2>>,
  cut: Vec<PointId>,
  cut_prev: Option<PointId>,
  uncut: Vec<PointId>,
  prev_shrink: Option<usize>,
  next_shrink: usize,
  done: bool,
}

impl<T: ValueTree> ShrinkablePolygon<T>
where
  <T as ValueTree>::Value: Clone + PolygonScalar,
{
  fn new(points: Vec<ShrinkablePoint<T, 2>>) -> Self {
    ShrinkablePolygon {
      points: points,
      cut: Vec::new(),
      cut_prev: None,
      uncut: Vec::new(),
      prev_shrink: None,
      next_shrink: 0,
      done: false,
    }
  }

  fn polygon(&self) -> Polygon<<T as ValueTree>::Value> {
    let mut poly = Polygon::new_unchecked(self.points.iter().map(|pt| pt.current()).collect());
    poly.rings[0].retain(|pid| !self.cut.contains(pid));
    poly
  }
}

impl<T: ValueTree> ValueTree for ShrinkablePolygon<T>
where
  <T as ValueTree>::Value: Clone + PolygonScalar,
{
  type Value = Polygon<<T as ValueTree>::Value>;

  fn current(&self) -> Self::Value {
    let mut poly = Polygon::new_unchecked(self.points.iter().map(|pt| pt.current()).collect());
    poly.rings[0].retain(|pid| !self.cut.contains(pid));
    Polygon::new_unchecked(
      poly.rings[0]
        .iter()
        .map(|pid| poly.points[pid.usize()].clone())
        .collect(),
    )
  }

  // Reduce the complexity and return 'true' if something changed.
  // Return 'false' if the complexity cannot be reduced.
  fn simplify(&mut self) -> bool {
    // if self.done {
    //   // eprintln!("Shrink done");
    //   return false;
    // }

    // If we previously cut an ear and the tests are still failing, make the cut permanent.
    // if let Some(cut) = self.cut {
    //   // eprintln!("Shrink cut {:?}", cut);
    //   self.points.remove(cut.usize());
    //   self.cut = None;
    //   self.uncut.clear();
    // }
    if self.cut_prev.is_some() {
      self.uncut.clear();
      self.cut_prev = None;
    }

    // Look for more ears to cut.
    let poly = self.polygon();
    if poly.rings[0].len() > 3 {
      for pt in poly.iter_boundary() {
        if !self.uncut.contains(&pt.point_id()) && pt.is_ear() {
          // eprintln!(
          //   "Cut ear: {:?} {}/{}",
          //   pt.point_id(),
          //   self.cut.len(),
          //   self.points.len()
          // );
          self.cut.push(pt.point_id());
          self.cut_prev = Some(pt.point_id());
          // if !self.uncut_prev.is_empty() {
          //   eprintln!("Re-trying: {:?}", self.uncut_prev);
          // }
          // self.uncut_prev.clear();
          // std::mem::swap(&mut self.uncut, &mut self.uncut_prev);
          return true;
        }
      }
    }

    // eprintln!("Simplify done: {:?} {:?}", self.uncut, self.uncut_prev);

    // No more ears can be cut. Let's try simplifying points:
    // eprintln!("Shrinking point: {}", self.next_shrink);
    let shrink_points = false;
    if shrink_points {
      while self.next_shrink < poly.rings[0].len()
        && !self.points[poly.rings[0][self.next_shrink].usize()].simplify()
      {
        self.next_shrink += 1;
        // eprintln!("Shrink next point: {}", self.shrink);
      }
      if self.next_shrink < poly.rings[0].len() {
        while self.polygon().validate().is_err() {
          // eprintln!("Bad point shrink. Undo: {}", self.next_shrink);
          if !self.points[poly.rings[0][self.next_shrink].usize()].complicate() {
            // eprintln!("Cannot undo. Abort");
            self.next_shrink = usize::MAX;
            return true;
          }
        }
        // eprintln!("Phew. Fixed: {}", self.next_shrink);
        self.prev_shrink = Some(self.next_shrink);
        return true;
      } else {
        self.done = true;
        return false;
      }
    } else {
      self.done = true;
      return false;
    }
    // return false;
  }

  // The value has been shrunk so much that the test-cases no longer fail.
  // Undo the last shrink action and find another way to shrink the value.
  fn complicate(&mut self) -> bool {
    if self.done {
      return false;
    }

    if let Some(cut) = self.cut_prev {
      self.cut.pop();
      self.cut_prev = None;
      // eprintln!(
      //   "Undo cut: {:?} {}/{}",
      //   cut,
      //   self.cut.len(),
      //   self.points.len()
      // );
      // self.done = true;
      // std::mem::swap(&mut self.uncut, &mut self.uncut_prev);
      self.uncut.push(cut);
      return true;
    }

    if let Some(idx) = self.prev_shrink {
      let key = self.polygon().rings[0][idx].usize();
      self.points[key].complicate();
      while self.polygon().validate().is_err() {
        // eprintln!("Bad point unshrink. Undo: {}", self.shrink);
        if !self.points[key].complicate() {
          // eprintln!("Cannot undo. Abort");
          self.done = true;
          return true;
        }
      }
      self.prev_shrink = None;
    }

    // if self.shrink < self.points.len() {
    //   // eprintln!("Undo shrink");
    //   if !self.points[self.shrink].complicate() {
    //     self.shrink += 1;
    //   } else {
    //     while self.polygon().validate().is_err() {
    //       // eprintln!("Bad point unshrink. Undo: {}", self.shrink);
    //       if !self.points[self.shrink].complicate() {
    //         // eprintln!("Cannot undo. Abort");
    //         self.done = true;
    //         return true;
    //       }
    //     }
    //   }
    //   return true;
    // } else {
    //   return false;
    // }
    return false;
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
    loop {
      let mut points = Vec::with_capacity(n);
      let mut set = BTreeSet::new();
      let mut actual = Vec::new();
      while actual.len() < n {
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
      // If all the points are colinear then two_opt_moves will fail.
      match crate::algorithms::two_opt_moves(actual, rng).map_err(|err| err.to_string()) {
        Err(_err) => continue,
        Ok(poly) => {
          assert_eq!(poly.rings[0].len(), points.len());
          // eprintln!("Re-ordering points");
          // FIXME: Super ugly:
          let mut new_points = Vec::new();
          for &pid in poly.rings[0].iter() {
            new_points.push(points[pid.usize()].clone());
          }

          return Ok(ShrinkablePolygon::new(new_points));
        }
      }
    }
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
      PolygonStrat(T::arbitrary_with(t_params), 3..50)
    } else {
      PolygonStrat(T::arbitrary_with(t_params), size_range)
    }
  }
}

// Arbitrary isn't defined for NotNan.
pub fn polygon_nn() -> impl Strategy<Value = Polygon<NotNan<f64>>> {
  PolygonStrat(
    any::<f64>().prop_filter_map("Check for NaN", |pt| rem_float(pt).try_into().ok()),
    3..50,
  )
}

pub fn polygon_big() -> impl Strategy<Value = Polygon<BigRational>> {
  PolygonStrat(
    any::<f64>().prop_filter_map("Check for NaN", |pt| BigRational::from_float(pt)),
    3..50,
  )
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary convex polygons

impl<T> Arbitrary for PolygonConvex<T>
where
  T: Bounded + PolygonScalar + SampleUniform + Copy + Into<BigInt>,
{
  type Strategy = Map<(Range<usize>, StrategyFor<u64>), fn(_: (usize, u64)) -> PolygonConvex<T>>;
  type Parameters = Range<usize>;
  fn arbitrary_with(mut range: Self::Parameters) -> Self::Strategy {
    if range.is_empty() {
      range = 3usize..100;
    }
    (range, any::<u64>()).prop_map(|(n, seed)| {
      let rng = &mut rand::rngs::SmallRng::seed_from_u64(seed);
      let p = PolygonConvex::random(n.max(3), rng);
      p
    })
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
  type Strategy = Mapped<Vec<T>, Point<T, N>>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    vec(any_with::<T>(params), N).prop_map(|vec: Vec<T>| Point {
      array: vec.try_into().unwrap(),
    })
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Vector

impl<T: Arbitrary, const N: usize> Arbitrary for Vector<T, N>
where
  T::Strategy: Clone,
  T::Parameters: Clone,
  T: Clone,
{
  type Strategy = Mapped<Point<T, N>, Vector<T, N>>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    Point::<T, N>::arbitrary_with(params).prop_map(|pt| pt.into())
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Direction

impl<T: Arbitrary, const N: usize> Arbitrary for Direction_<T, N>
where
  T::Strategy: Clone,
  T::Parameters: Clone,
  T: Clone,
{
  type Strategy = Mapped<(bool, Point<T, N>), Direction_<T, N>>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    (any::<bool>(), Point::<T, N>::arbitrary_with(params)).prop_map(|(is_pt, pt)| {
      if is_pt {
        Direction_::Through(pt)
      } else {
        Direction_::Vector(pt.into())
      }
    })
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Line

impl<T: Arbitrary, const N: usize> Arbitrary for Line_<T, N>
where
  T::Strategy: Clone,
  T::Parameters: Clone,
  T: Clone,
{
  type Strategy = Mapped<(Point<T, N>, Direction_<T, N>), Line_<T, N>>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(Point<T, N>, Direction_<T, N>)>((params.clone(), params))
      .prop_map(|(origin, direction)| Line_ { origin, direction })
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary LineSoS

impl<T: Arbitrary, const N: usize> Arbitrary for LineSoS_<T, N>
where
  T::Strategy: Clone,
  T::Parameters: Clone,
  T: Clone,
{
  type Strategy = Mapped<Line_<T, N>, LineSoS_<T, N>>;
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<Line_<T, N>>(params).prop_map(|line| line.into())
  }
}

///////////////////////////////////////////////////////////////////////////////
// Convenience functions

// FIXME: Move this impl to 'ordered_float' crate.
// impl Arbitrary for NotNan<f64> {
//   type Strategy = num::f64::Any;
//   type Parameters = ();
//   fn arbitrary_with(_params: ()) -> Self::Strategy {
//     POSITIVE | NEGATIVE | NORMAL | SUBNORMAL | ZERO;
//   }
// }

// Arbitrary isn't defined for NotNan.
pub fn any_nn<const N: usize>() -> impl Strategy<Value = Point<NotNan<f64>, N>> {
  any::<Point<f64, N>>().prop_filter_map("Check for NaN", |pt| pt.map(rem_float).try_into().ok())
}

// Float representation: mantissa * 2^exponent * sign
// This function changes the exponent modulo 250. This rules out extreme
// numbers (very large, very small, very close to zero). Such extremes
// are likely to overflow since the arbitrary precision machinery we're
// using cannot compute answers with an exponent larger than 1024.
fn rem_float(f: f64) -> f64 {
  let (mantissa, exponent, sign) = f.integer_decode();
  ((mantissa as f64) * 2f64.powi(exponent as i32 % 250)).copysign(sign as f64)
}

// Arbitrary isn't defined for BigInt.
pub fn any_r<const N: usize>() -> impl Strategy<Value = Point<BigInt, N>> {
  any::<Point<isize, N>>().prop_map(|pt| pt.cast())
}

// pub fn any_64<const N: usize>() -> impl Strategy<Value = Point<i64, N>> {
//   any::<Point<i64, N>>()
// }

// pub fn any_8<const N: usize>() -> impl Strategy<Value = Point<i8, N>> {
//   any::<Point<i8, N>>()
// }

///////////////////////////////////////////////////////////////////////////////
// Arbitrary triangle

impl<T> Arbitrary for Triangle<T>
where
  T: Arbitrary + Clone + PolygonScalar,
  <T as Arbitrary>::Strategy: Clone,
  <T as Arbitrary>::Parameters: Clone,
{
  type Strategy = FilterMapped<[Point<T, 2>; 3], Triangle<T>>;
  type Parameters = <Point<T, 2> as Arbitrary>::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<[Point<T, 2>; 3]>(params)
      .prop_filter_map("Ensure CCW", |pts| Triangle::new(pts).ok())
  }
}
