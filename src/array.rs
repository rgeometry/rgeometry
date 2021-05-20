use array_init::array_init;
use num_traits::NumOps;
use num_traits::NumRef;
use std::cmp::Ordering;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

pub fn raw_arr_sub<T, const N: usize>(lhs: &[T; N], rhs: &[T; N]) -> [T; N]
where
  for<'a> &'a T: Sub<Output = T>,
{
  array_init(|i| (lhs.index(i) - rhs.index(i)))
}

#[derive(PartialEq, Debug)]
pub enum Turn {
  CounterClockWise,
  ClockWise,
  CoLinear,
}

// How does the line from (0,0) to q to r turn?
pub fn raw_arr_turn_origin<T>(q: &[T; 2], r: &[T; 2]) -> Turn
where
  T: PartialOrd,
  for<'a> &'a T: Mul<Output = T>,
{
  let [ux, uy] = q;
  let [vx, vy] = r;
  match (ux * vy).partial_cmp(&(uy * vx)).unwrap_or(Ordering::Equal) {
    Ordering::Less => Turn::ClockWise,
    Ordering::Greater => Turn::CounterClockWise,
    Ordering::Equal => Turn::CoLinear,
  }
}

pub fn raw_arr_turn<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Turn
where
  T: Mul<T, Output = T> + PartialOrd,
  for<'a> &'a T: Sub<Output = T>,
{
  let [ux, uy] = raw_arr_sub(q, p);
  let [vx, vy] = raw_arr_sub(r, p);
  match (ux * vy).partial_cmp(&(uy * vx)).unwrap_or(Ordering::Equal) {
    Ordering::Less => Turn::ClockWise,
    Ordering::Greater => Turn::CounterClockWise,
    Ordering::Equal => Turn::CoLinear,
  }
}

// Sort 'p' and 'q' counterclockwise around (0,0) along the 'z' axis.
pub fn ccw_cmp_around_origin_with<T>(z: &[T; 2], p: &[T; 2], q: &[T; 2]) -> Ordering
where
  T: Clone + PartialOrd,
  for<'a> &'a T: Mul<Output = T> + Neg<Output = T>,
{
  let [zx, zy] = z;
  let b: &[T; 2] = &[zy.neg(), zx.clone()];
  let ap = raw_arr_turn_origin(z, p);
  let aq = raw_arr_turn_origin(z, q);
  let on_zero = |d: &[T; 2]| match raw_arr_turn_origin(b, d) {
    Turn::CounterClockWise => false,
    Turn::ClockWise => true,
    Turn::CoLinear => true,
  };
  let cmp = match raw_arr_turn_origin(p, q) {
    Turn::CounterClockWise => Ordering::Less,
    Turn::ClockWise => Ordering::Greater,
    Turn::CoLinear => Ordering::Equal,
  };
  match (ap, aq) {
    (Turn::CounterClockWise, Turn::CounterClockWise) => cmp,
    (Turn::CounterClockWise, Turn::ClockWise) => Ordering::Less,
    (Turn::CounterClockWise, Turn::CoLinear) => {
      if on_zero(q) {
        Ordering::Greater
      } else {
        Ordering::Less
      }
    }

    (Turn::ClockWise, Turn::CounterClockWise) => Ordering::Greater,
    (Turn::ClockWise, Turn::ClockWise) => cmp,
    (Turn::ClockWise, Turn::CoLinear) => Ordering::Less,

    (Turn::CoLinear, Turn::CounterClockWise) => {
      if on_zero(p) {
        Ordering::Less
      } else {
        Ordering::Greater
      }
    }
    (Turn::CoLinear, Turn::ClockWise) => Ordering::Less,
    (Turn::CoLinear, Turn::CoLinear) => match (on_zero(p), on_zero(q)) {
      (true, true) => Ordering::Equal,
      (false, false) => Ordering::Equal,
      (true, false) => Ordering::Less,
      (false, true) => Ordering::Greater,
    },
  }
}
