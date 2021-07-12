// #![deny(warnings)]
#![doc(test(no_crate_inject))]
#![allow(unused_imports)]
// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]
#![doc(html_playground_url = "https://rgeometry.org/playground.html")]
#![doc(test(no_crate_inject))]
use num_traits::*;
use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::*;

pub mod algorithms;
pub mod data;
mod intersection;
mod matrix;
mod orientation;
mod transformation;
mod utils;

pub use orientation::{Orientation, SoS};

pub use intersection::Intersects;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  InsufficientVertices,
  SelfIntersections,
  DuplicatePoints,
  /// Two consecutive line segments are either colinear or oriented clockwise.
  ConvexViolation,
  ClockWiseViolation,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      Error::InsufficientVertices => write!(f, "Insufficient vertices"),
      Error::SelfIntersections => write!(f, "Self intersections"),
      Error::DuplicatePoints => write!(f, "Duplicate points"),
      Error::ConvexViolation => write!(f, "Convex violation"),
      Error::ClockWiseViolation => write!(f, "Clockwise violation"),
    }
  }
}

// FIXME: Should include ZHashable.
pub trait PolygonScalar:
  std::fmt::Debug + Neg<Output = Self> + NumAssignOps + NumOps<Self, Self> + Ord + Sum + Clone
{
  fn from_constant(val: i8) -> Self;
  fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_vector_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_perp_vector_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
}

macro_rules! fixed_precision {
  ( $ty:ty, $uty:ty, $long:ty, $ulong: ty ) => {
    impl PolygonScalar for $ty {
      fn from_constant(val: i8) -> Self {
        val as $ty
      }
      fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        fn diff(a: $ty, b: $ty) -> $ulong {
          if b > a {
            b.wrapping_sub(a) as $uty as $ulong
          } else {
            a.wrapping_sub(b) as $uty as $ulong
          }
        }
        let pq_x = diff(p[0], q[0]);
        let pq_y = diff(p[1], q[1]);
        let (pq_dist_squared, pq_overflow) = (pq_x * pq_x).overflowing_add(pq_y * pq_y);
        let pr_x = diff(p[0], r[0]);
        let pr_y = diff(p[1], r[1]);
        let (pr_dist_squared, pr_overflow) = (pr_x * pr_x).overflowing_add(pr_y * pr_y);
        match (pq_overflow, pr_overflow) {
          (true, false) => Ordering::Greater,
          (false, true) => Ordering::Less,
          _ => pq_dist_squared.cmp(&pr_dist_squared),
        }
      }

      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        // Return the absolute difference along with its sign.
        // diff(0, 10) => (10, true)
        // diff(10, 0) => (10, false)
        // diff(i8::MIN,i8:MAX) => (255_u16, true)
        // diff(a,b) = (c, sign) where a = if sign { b-c } else { b+c }
        fn diff(a: $ty, b: $ty) -> ($ulong, bool) {
          if b > a {
            (b.wrapping_sub(a) as $uty as $ulong, true)
          } else {
            (a.wrapping_sub(b) as $uty as $ulong, false)
          }
        }
        let (ux, ux_neg) = diff(q[0], p[0]);
        let (vy, vy_neg) = diff(r[1], p[1]);
        let ux_vy_neg = ux_neg.bitxor(vy_neg) && ux != 0 && vy != 0;
        let (uy, uy_neg) = diff(q[1], p[1]);
        let (vx, vx_neg) = diff(r[0], p[0]);
        let uy_vx_neg = uy_neg.bitxor(vx_neg) && uy != 0 && vx != 0;
        match (ux_vy_neg, uy_vx_neg) {
          (true, false) => Ordering::Less,
          (false, true) => Ordering::Greater,
          (true, true) => (uy * vx).cmp(&(ux * vy)),
          (false, false) => (ux * vy).cmp(&(uy * vx)),
        }
      }

      fn cmp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        // Return the absolute difference along with its sign.
        // diff(0, 10) => (10, true)
        // diff(10, 0) => (10, false)
        // diff(i8::MIN,i8:MAX) => (255_u16, true)
        // diff(a,b) = (c, sign) where a = if sign { b-c } else { b+c }
        fn diff(a: $ty, b: $ty) -> ($ulong, bool) {
          if b > a {
            (b.wrapping_sub(a) as $uty as $ulong, true)
          } else {
            (a.wrapping_sub(b) as $uty as $ulong, false)
          }
        }
        let (ux, ux_neg) = (vector[0].unsigned_abs() as $ulong, vector[0] < 0);
        let (vy, vy_neg) = diff(q[1], p[1]);
        // neg xor neg = pos = 0
        // neg xor pos = neg = 1
        // pos xor neg = neg = 1
        // pos xor pos = pos = 0
        let ux_vy_neg = ux_neg.bitxor(vy_neg) && ux != 0 && vy != 0;
        let (uy, uy_neg) = (vector[1].unsigned_abs() as $ulong, vector[1] < 0);
        let (vx, vx_neg) = diff(q[0], p[0]);
        let uy_vx_neg = uy_neg.bitxor(vx_neg) && uy != 0 && vx != 0;
        match (ux_vy_neg, uy_vx_neg) {
          (true, false) => Ordering::Less,
          (false, true) => Ordering::Greater,
          (true, true) => (uy * vx).cmp(&(ux * vy)),
          (false, false) => (ux * vy).cmp(&(uy * vx)),
        }
      }

      fn cmp_perp_vector_slope(
        vector: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // Return the absolute difference along with its sign.
        // diff(0, 10) => (10, true)
        // diff(10, 0) => (10, false)
        // diff(i8::MIN,i8:MAX) => (255_u16, true)
        // diff(a,b) = (c, sign) where a = if sign { b-c } else { b+c }
        fn diff(a: $ty, b: $ty) -> ($ulong, bool) {
          if b > a {
            (b.wrapping_sub(a) as $uty as $ulong, true)
          } else {
            (a.wrapping_sub(b) as $uty as $ulong, false)
          }
        }
        let (ux, ux_neg) = (vector[1].unsigned_abs() as $ulong, vector[1] > 0);
        let (vy, vy_neg) = diff(q[1], p[1]);
        // neg xor neg = pos = 0
        // neg xor pos = neg = 1
        // pos xor neg = neg = 1
        // pos xor pos = pos = 0
        let ux_vy_neg = ux_neg.bitxor(vy_neg) && ux != 0 && vy != 0;
        let (uy, uy_neg) = (vector[0].unsigned_abs() as $ulong, vector[0] < 0);
        let (vx, vx_neg) = diff(q[0], p[0]);
        let uy_vx_neg = uy_neg.bitxor(vx_neg) && uy != 0 && vx != 0;
        match (ux_vy_neg, uy_vx_neg) {
          (true, false) => Ordering::Less,
          (false, true) => Ordering::Greater,
          (true, true) => (uy * vx).cmp(&(ux * vy)),
          (false, false) => (ux * vy).cmp(&(uy * vx)),
        }
      }
    }
  };
}

macro_rules! arbitrary_precision {
  ( $( $ty:ty ),* ) => {
    $(
      impl PolygonScalar for $ty {
      fn from_constant(val: i8) -> Self {
        <$ty>::from_i8(val).unwrap()
      }
      fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        let pq_x = &p[0] - &q[0];
        let pq_y = &p[1] - &q[1];
        let pq_dist_squared: Self = &pq_x*&pq_x + &pq_y*&pq_y;
        let pr_x = &p[0] - &r[0];
        let pr_y = &p[1] - &r[1];
        let pr_dist_squared: Self = &pr_x*&pr_x + &pr_y*&pr_y;
        pq_dist_squared.cmp(&pr_dist_squared)
      }

      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        let slope1 = (&r[1] - &q[1]) * (&q[0] - &p[0]);
        let slope2 = (&q[1] - &p[1]) * (&r[0] - &q[0]);
        slope1.cmp(&slope2)
      }
      fn cmp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_slope(
          p,
          &[&p[0] + &vector[0], &p[1] + &vector[1]],
          q
        )
      }
      fn cmp_perp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_slope(
          p,
          &[&p[0] - &vector[1], &p[1] + &vector[0]],
          q
        )
      }
    })*
  };
}

fixed_precision!(i8, u8, i16, u16);
fixed_precision!(i16, u16, i32, u32);
fixed_precision!(i32, u32, i64, u64);
fixed_precision!(i64, u64, i128, u128);
fixed_precision!(isize, usize, i128, u128);
arbitrary_precision!(num_bigint::BigInt);
arbitrary_precision!(num_rational::BigRational);
arbitrary_precision!(ordered_float::OrderedFloat<f32>);
arbitrary_precision!(ordered_float::OrderedFloat<f64>);
arbitrary_precision!(ordered_float::NotNan<f32>);
arbitrary_precision!(ordered_float::NotNan<f64>);

#[cfg(feature = "rug")]
impl PolygonScalar for rug::Integer {
  fn from_constant(val: i8) -> Self {
    rug::Integer::from(val)
  }
  fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
    let [qx, qy] = q.clone();
    let [px, py] = p.clone();
    let pq_x = px - qx;
    let pq_y = py - qy;
    let pq_dist_squared: Self = pq_x.square() + pq_y.square();
    let [rx, ry] = r.clone();
    let [px, py] = p.clone();
    let pr_x = px - rx;
    let pr_y = py - ry;
    let pr_dist_squared: Self = pr_x.square() + pr_y.square();
    pq_dist_squared.cmp(&pr_dist_squared)
  }

  fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
    let [qx, qy] = q.clone();
    let [rx, ry] = r.clone();
    let ry_qy = ry - &q[1];
    let qx_px = qx - &p[0];
    let slope1 = ry_qy * qx_px;
    let qy_py = qy - &p[1];
    let rx_qx = rx - &q[0];
    let slope2 = qy_py * rx_qx;
    slope1.cmp(&slope2)
  }
  fn cmp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
    let new_x = rug::Integer::from(&p[0] + &vector[0]);
    let new_y = rug::Integer::from(&p[1] + &vector[1]);
    PolygonScalar::cmp_slope(p, &[new_x, new_y], q)
  }
  fn cmp_perp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
    let new_x = rug::Integer::from(&p[0] + &vector[1]);
    let new_y = rug::Integer::from(&p[1] + &vector[0]);
    PolygonScalar::cmp_slope(p, &[new_x, new_y], q)
  }
}

#[cfg(test)]
pub mod testing;
