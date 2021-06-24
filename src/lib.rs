#![doc(test(no_crate_inject))]
#![allow(unused_imports)]
// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]
#![doc(html_playground_url = "https://rgeometry.org/rgeometry-playground/")]
#![doc(test(no_crate_inject))]
use num_traits::*;
use std::iter::Sum;
use std::ops::*;

pub mod algorithms;
mod array;
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
{
}

pub trait PolygonScalarRef<T = Self, Output = Self>: Clone + NumOps<T, Output> {}
impl<T, Rhs, Output> PolygonScalarRef<Rhs, Output> for T where T: Clone + NumOps<Rhs, Output> {}

pub trait Extended: Signed {
  type ExtendedSigned: Signed + Clone;
  type ExtendedUnsigned: Clone;
  fn extend_signed(self) -> Self::ExtendedSigned;
  fn extend_unsigned(self) -> Self::ExtendedUnsigned;
}

impl Extended for i32 {
  type ExtendedSigned = i64;
  type ExtendedUnsigned = u64;
  fn extend_signed(self) -> Self::ExtendedSigned {
    self as Self::ExtendedSigned
  }
  fn extend_unsigned(self) -> Self::ExtendedUnsigned {
    self as Self::ExtendedUnsigned
  }
}

impl Extended for num_bigint::BigInt {
  type ExtendedSigned = num_bigint::BigInt;
  type ExtendedUnsigned = num_bigint::BigInt;
  fn extend_signed(self) -> Self::ExtendedSigned {
    self
  }
  fn extend_unsigned(self) -> Self::ExtendedUnsigned {
    self
  }
}
