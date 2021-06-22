use array_init::array_init;
use std::cmp::Ordering;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

pub fn raw_arr_sub<T, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N]
where
  T: Sub<Output = T> + Clone,
{
  array_init(|i| (lhs.index(i).clone() - rhs.index(i).clone()))
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum Orientation {
  CounterClockWise,
  ClockWise,
  CoLinear,
}
use Orientation::*;

impl Orientation {
  pub fn new<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
  {
    raw_arr_turn(p, q, r)
  }

  pub fn is_colinear<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
  {
    Orientation::new(p, q, r) == Orientation::CoLinear
  }

  pub fn is_ccw<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
  {
    Orientation::new(p, q, r) == Orientation::CounterClockWise
  }

  pub fn is_cw<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
  {
    Orientation::new(p, q, r) == Orientation::ClockWise
  }

  pub fn around_origin<T>(q: &[T; 2], r: &[T; 2]) -> Orientation
  where
    T: Ord + Mul<Output = T> + Clone,
  {
    raw_arr_turn_origin(q, r)
  }

  pub fn reverse(self) -> Orientation {
    match self {
      Orientation::CounterClockWise => Orientation::ClockWise,
      Orientation::ClockWise => Orientation::CounterClockWise,
      Orientation::CoLinear => Orientation::CoLinear,
    }
  }
}

// How does the line from (0,0) to q to r turn?
pub fn raw_arr_turn_origin<T>(q: &[T; 2], r: &[T; 2]) -> Orientation
where
  T: Ord + Mul<Output = T> + Clone,
  // for<'a> &'a T: Mul<Output = T>,
{
  let [ux, uy] = q.clone();
  let [vx, vy] = r.clone();
  match (ux * vy).cmp(&(uy * vx)) {
    Ordering::Less => ClockWise,
    Ordering::Greater => CounterClockWise,
    Ordering::Equal => CoLinear,
  }
}

pub fn raw_arr_turn<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
where
  T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
  // for<'a> &'a T: Sub<Output = T>,
{
  let [ux, uy] = raw_arr_sub(q, p);
  let [vx, vy] = raw_arr_sub(r, p);
  match (ux * vy).cmp(&(uy * vx)) {
    Ordering::Less => ClockWise,
    Ordering::Greater => CounterClockWise,
    Ordering::Equal => CoLinear,
  }
}

// Sort 'p' and 'q' counterclockwise around (0,0) along the 'z' axis.
pub fn ccw_cmp_around_origin_with<T>(z: &[T; 2], p: &[T; 2], q: &[T; 2]) -> Ordering
where
  T: Clone + Ord + Mul<Output = T> + Neg<Output = T>,
{
  let [zx, zy] = z;
  let b: &[T; 2] = &[zy.clone().neg(), zx.clone()];
  let ap = raw_arr_turn_origin(z, p);
  let aq = raw_arr_turn_origin(z, q);
  let on_zero = |d: &[T; 2]| match raw_arr_turn_origin(b, d) {
    CounterClockWise => false,
    ClockWise => true,
    CoLinear => true,
  };
  let cmp = match raw_arr_turn_origin(p, q) {
    CounterClockWise => Ordering::Less,
    ClockWise => Ordering::Greater,
    CoLinear => Ordering::Equal,
  };
  match (ap, aq) {
    (CounterClockWise, CounterClockWise) => cmp,
    (CounterClockWise, ClockWise) => Ordering::Less,
    (CounterClockWise, CoLinear) => {
      if on_zero(q) {
        Ordering::Greater
      } else {
        Ordering::Less
      }
    }

    (ClockWise, CounterClockWise) => Ordering::Greater,
    (ClockWise, ClockWise) => cmp,
    (ClockWise, CoLinear) => Ordering::Less,

    (CoLinear, CounterClockWise) => {
      if on_zero(p) {
        Ordering::Less
      } else {
        Ordering::Greater
      }
    }
    (CoLinear, ClockWise) => Ordering::Less,
    (CoLinear, CoLinear) => match (on_zero(p), on_zero(q)) {
      (true, true) => Ordering::Equal,
      (false, false) => Ordering::Equal,
      (true, false) => Ordering::Less,
      (false, true) => Ordering::Greater,
    },
  }
}
