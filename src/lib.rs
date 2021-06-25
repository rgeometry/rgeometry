#![doc(test(no_crate_inject))]
#![allow(unused_imports)]
// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]
#![doc(html_playground_url = "https://rgeometry.org/rgeometry-playground/")]
#![doc(test(no_crate_inject))]
use num_traits::*;
use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::*;

pub mod algorithms;
pub mod array;
pub mod data;
mod intersection;
mod matrix;
mod transformation;
mod utils;

pub use array::Orientation;

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
pub trait PolygonScalar<T = Self, Output = Self>:
  PolygonScalarRef<T, Output>
  + AddAssign<Output>
  + MulAssign<Output>
  + FromPrimitive
  + One
  + Zero
  + Sum
  + Ord
  + Neg<Output = Self>
  + Signed
  + std::fmt::Debug
  + Extended
{
}
impl<T, Rhs, Output> PolygonScalar<Rhs, Output> for T where
  T: PolygonScalarRef<Rhs, Output>
    + AddAssign<Output>
    + MulAssign<Output>
    + FromPrimitive
    + One
    + Zero
    + Sum
    + Ord
    + Neg<Output = Self>
    + Signed
    + std::fmt::Debug
    + Extended
{
}

pub trait PolygonScalarRef<T = Self, Output = Self>: Clone + NumOps<T, Output> {}
impl<T, Rhs, Output> PolygonScalarRef<Rhs, Output> for T where T: Clone + NumOps<Rhs, Output> {}

pub trait Extended: NumOps<Self, Self> + Ord + Clone {
  type ExtendedSigned: Clone + NumOps<Self::ExtendedSigned, Self::ExtendedSigned> + Ord;
  fn extend_signed(self) -> Self::ExtendedSigned;
  fn truncate_signed(val: Self::ExtendedSigned) -> Self;
  fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
}

macro_rules! fixed_precision {
  ( $ty:ty, $uty:ty, $long:ty, $ulong: ty ) => {
    impl Extended for $ty {
      type ExtendedSigned = $long;
      fn extend_signed(self) -> Self::ExtendedSigned {
        self as Self::ExtendedSigned
      }
      fn truncate_signed(val: Self::ExtendedSigned) -> Self {
        val as Self
      }

      // ux = 0 - 1 = 1, true
      // vy = 8 - 0 = 8, false
      // uy = 6 - 0 = 6, false
      // vx = 0 - 1 = 1, true

      // ux*vy = -8
      // uy*vx = -6

      // P: 1 0
      // Q: 0 6
      // R: 0 8
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
        // neg xor neg = pos = 0
        // neg xor pos = neg = 1
        // pos xor neg = neg = 1
        // pos xor pos = pos = 0
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
        // uy_vx_neg
        //   .cmp(&ux_vy_neg)
        //   .then_with(|| (ux * vy).cmp(&(uy * vx)))
      }
    }
  };
}

macro_rules! arbitrary_precision {
  ( $( $ty:ty ),* ) => {
    $(
      impl Extended for $ty {
      type ExtendedSigned = $ty;
      fn extend_signed(self) -> Self::ExtendedSigned {
        self
      }
      fn truncate_signed(val: Self::ExtendedSigned) -> Self {
        val
      }
      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        let slope1 = (r[1].clone() - q[1].clone()) * (q[0].clone() - p[0].clone());
        let slope2 = (q[1].clone() - p[1].clone()) * (r[0].clone() - q[0].clone());
        slope1.cmp(&slope2)
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
arbitrary_precision!(ordered_float::NotNan<f32>);
