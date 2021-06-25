use array_init::array_init;
use num_traits::CheckedMul;
use num_traits::Signed;
use std::cmp::Ordering;
use std::ops::BitXor;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use crate::Extended;

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
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord + Extended,
  {
    // raw_arr_turn(p, q, r)
    match Extended::cmp_slope(p, q, r) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  pub fn is_colinear<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord + Extended,
  {
    Orientation::new(p, q, r) == Orientation::CoLinear
  }

  pub fn is_ccw<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord + Extended,
  {
    Orientation::new(p, q, r) == Orientation::CounterClockWise
  }

  pub fn is_cw<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> bool
  where
    T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord + Extended,
  {
    Orientation::new(p, q, r) == Orientation::ClockWise
  }

  pub fn around_origin<T>(q: &[T; 2], r: &[T; 2]) -> Orientation
  where
    T: Ord + Mul<Output = T> + Clone + Extended,
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

// pub fn raw_arr_turn<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
// where
//   T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
//   // for<'a> &'a T: Sub<Output = T>,
// {
//   let [ux, uy] = raw_arr_sub(q, p);
//   let [vx, vy] = raw_arr_sub(r, p);
//   match (ux * vy).cmp(&(uy * vx)) {
//     Ordering::Less => ClockWise,
//     Ordering::Greater => CounterClockWise,
//     Ordering::Equal => CoLinear,
//   }
// }

// pub fn raw_arr_turn_2<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
// where
//   T: Clone + Mul<T, Output = T> + Sub<Output = T> + Ord,
//   // for<'a> &'a T: Sub<Output = T>,
// {
//   let slope1 = (q[1].clone() - p[1].clone()) * (r[0].clone() - q[0].clone());
//   let slope2 = (r[1].clone() - q[1].clone()) * (q[0].clone() - p[0].clone());
//   match slope1.cmp(&slope2) {
//     Ordering::Less => ClockWise,
//     Ordering::Greater => CounterClockWise,
//     Ordering::Equal => CoLinear,
//   }
// }

// pub fn extended_orientation_2<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
// where
//   T: Extended,
// {
//   // let slope1 = (q[1].clone() - p[1].clone()) * (r[0].clone() - q[0].clone());
//   // let slope2 = (r[1].clone() - q[1].clone()) * (q[0].clone() - p[0].clone());
//   let ux = q[0].clone().extend_signed() - p[0].clone().extend_signed();
//   let uy = q[1].clone().extend_signed() - p[1].clone().extend_signed();
//   let vx = r[0].clone().extend_signed() - p[0].clone().extend_signed();
//   let vy = r[1].clone().extend_signed() - p[1].clone().extend_signed();
//   let ux_vy_signum = ux.signum() * vy.signum();
//   let uy_vx_signum = uy.signum() * vx.signum();
//   match ux_vy_signum.cmp(&uy_vx_signum).then_with(|| {
//     (ux.do_unsigned_abs() * vy.do_unsigned_abs())
//       .cmp(&(uy.do_unsigned_abs() * vx.do_unsigned_abs()))
//   }) {
//     Ordering::Less => ClockWise,
//     Ordering::Greater => CounterClockWise,
//     Ordering::Equal => CoLinear,
//   }
// }

// #[inline(never)]
// pub fn extended_orientation_3<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Orientation
// where
//   T: Extended,
// {
//   // let slope1 = (q[1].clone() - p[1].clone()) * (r[0].clone() - q[0].clone());
//   // let slope2 = (r[1].clone() - q[1].clone()) * (q[0].clone() - p[0].clone());
//   let (ux, ux_neg) = q[0].clone().diff(p[0].clone());
//   let (vy, vy_neg) = r[1].clone().diff(p[1].clone());
//   let ux_vy_signum = ux_neg.bitxor(vy_neg);
//   let (uy, uy_neg) = q[1].clone().diff(p[1].clone());
//   let (vx, vx_neg) = r[0].clone().diff(p[0].clone());
//   let uy_vx_signum = uy_neg.bitxor(vx_neg);
//   match uy_vx_signum
//     .cmp(&ux_vy_signum)
//     .then_with(|| (ux * vy).cmp(&(uy * vx)))
//   {
//     Ordering::Less => ClockWise,
//     Ordering::Greater => CounterClockWise,
//     Ordering::Equal => CoLinear,
//   }
// }

pub fn extended_orientation_i64(p: &[i64; 2], q: &[i64; 2], r: &[i64; 2]) -> Ordering {
  crate::Extended::cmp_slope(p, q, r)
}

// pub fn turn_i64(p: &[i64; 2], q: &[i64; 2], r: &[i64; 2]) -> Orientation {
//   raw_arr_turn_2(p, q, r)
// }

// pub fn turn_bigint(p: &[i64; 2], q: &[i64; 2], r: &[i64; 2]) -> Orientation {
//   use num_bigint::*;
//   let slope1: BigInt =
//     (BigInt::from(q[1]) - BigInt::from(p[1])) * (BigInt::from(r[0]) - BigInt::from(q[0]));
//   let slope2: BigInt =
//     (BigInt::from(r[1]) - BigInt::from(q[1])) * (BigInt::from(q[0]) - BigInt::from(p[0]));
//   match slope1.cmp(&slope2) {
//     Ordering::Less => ClockWise,
//     Ordering::Greater => CounterClockWise,
//     Ordering::Equal => CoLinear,
//   }
// }

// pub fn turn_i64_fast_or_slow(p: &[i64; 2], q: &[i64; 2], r: &[i64; 2]) -> Orientation {
//   turn_t_fast_path(p, q, r).unwrap_or_else(|| extended_orientation_3(p, q, r))
// }

// pub fn turn_t_fast_path<T>(p: &[T; 2], q: &[T; 2], r: &[T; 2]) -> Option<Orientation>
// where
//   T: Extended,
// {
//   // let slope1 = (q[1].clone().checked_sub(&p[1].clone())?)
//   //   .checked_mul(&r[0].clone().checked_sub(&q[0].clone())?)?
//   //   .extend_signed();
//   // let slope2 = (r[1].clone().checked_sub(&q[1].clone())?)
//   //   .checked_mul(&q[0].clone().checked_sub(&p[0].clone())?)?
//   //   .extend_signed();
//   let slope1 = (q[1].clone().checked_sub(&p[1].clone())?.extend_signed())
//     .checked_mul(&r[0].clone().checked_sub(&q[0].clone())?.extend_signed());
//   let slope2 = (r[1].clone().checked_sub(&q[1].clone())?.extend_signed())
//     .checked_mul(&q[0].clone().checked_sub(&p[0].clone())?.extend_signed());
//   match slope1.cmp(&slope2) {
//     Ordering::Less => Some(ClockWise),
//     Ordering::Greater => Some(CounterClockWise),
//     Ordering::Equal => Some(CoLinear),
//   }
// }

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn orientation_limit_1() {
    Extended::cmp_slope(
      &[i8::MAX, i8::MAX],
      &[i8::MIN, i8::MIN],
      &[i8::MIN, i8::MIN],
    );
  }

  #[test]
  fn cmp_slope_1() {
    assert_eq!(
      Extended::cmp_slope(&[0i8, 0], &[1, 1], &[2, 2],),
      Ordering::Equal
    );
  }
  #[test]
  fn cmp_slope_2() {
    assert_eq!(
      Orientation::new(&[0i8, 0], &[0, 1], &[2, 2],),
      Orientation::ClockWise
    );
  }

  #[test]
  fn orientation_limit_2() {
    let slice = &[i8::MIN, i8::MAX, 0, -10, 10];
    for &coord1 in slice {
      for &coord2 in slice {
        for &coord3 in slice {
          for &coord4 in slice {
            for &coord5 in slice {
              for &coord6 in slice {
                dbg!(coord1, coord2, coord3, coord4, coord5, coord6);
                Extended::cmp_slope(&[coord1, coord2], &[coord3, coord4], &[coord5, coord6]);
              }
            }
          }
        }
      }
    }
  }
}
