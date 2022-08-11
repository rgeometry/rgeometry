// #![deny(warnings)]
#![deny(clippy::cast_lossless)]
#![doc(test(no_crate_inject))]
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
  CoLinearViolation,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      Error::InsufficientVertices => write!(f, "Insufficient vertices"),
      Error::SelfIntersections => write!(f, "Self intersections"),
      Error::DuplicatePoints => write!(f, "Duplicate points"),
      Error::ConvexViolation => write!(f, "Convex violation"),
      Error::ClockWiseViolation => write!(f, "Clockwise violation"),
      Error::CoLinearViolation => write!(
        f,
        "Two or more points are colinear and no valid solution exists"
      ),
    }
  }
}

pub trait TotalOrd {
  fn total_cmp(&self, other: &Self) -> Ordering;

  fn total_min(self, other: Self) -> Self
  where
    Self: Sized,
  {
    std::cmp::min_by(self, other, TotalOrd::total_cmp)
  }

  fn total_max(self, other: Self) -> Self
  where
    Self: Sized,
  {
    std::cmp::max_by(self, other, TotalOrd::total_cmp)
  }
}

impl TotalOrd for u32 {
  fn total_cmp(&self, other: &Self) -> Ordering {
    self.cmp(other)
  }
}

impl<A: TotalOrd> TotalOrd for &A {
  fn total_cmp(&self, other: &Self) -> Ordering {
    (*self).total_cmp(*other)
  }
}

impl<A: TotalOrd, B: TotalOrd> TotalOrd for (A, B) {
  fn total_cmp(&self, other: &Self) -> Ordering {
    self
      .0
      .total_cmp(&other.0)
      .then_with(|| self.1.total_cmp(&other.1))
  }
}

// FIXME: Should include ZHashable.
pub trait PolygonScalar:
  std::fmt::Debug
  + Neg<Output = Self>
  + NumAssignOps
  + NumOps<Self, Self>
  + TotalOrd
  + PartialOrd
  + Sum
  + Clone
{
  fn from_constant(val: i8) -> Self;
  fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_vector_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
  fn cmp_perp_vector_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering;
}

macro_rules! fixed_precision {
  ( $ty:ty, $uty:ty, $long:ty, $ulong: ty ) => {
    impl TotalOrd for $ty {
      fn total_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
      }
    }

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
      impl TotalOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Ordering {
          self.cmp(other)
        }
      }

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

macro_rules! wrapped_floating_precision {
  ( $( $ty:ty ),* ) => {
    $(
      impl TotalOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Ordering {
          self.cmp(other)
        }
      }

      impl PolygonScalar for $ty {
      fn from_constant(val: i8) -> Self {
        <$ty>::from_i8(val).unwrap()
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_dist(
          &[float_to_rational(p[0].into_inner()), float_to_rational(p[1].into_inner())],
          &[float_to_rational(q[0].into_inner()), float_to_rational(q[1].into_inner())],
          &[float_to_rational(r[0].into_inner()), float_to_rational(r[1].into_inner())],
        )
      }

      // This function uses the arbitrary precision machinery of `geometry_predicates` to
      // quickly compute the orientation of three 2D points. This is about 10x-50x slower
      // than the inexact version.
      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        let orient = geometry_predicates::predicates::orient2d(
          [p[0].into_inner() as f64, p[1].into_inner() as f64],
          [q[0].into_inner() as f64, q[1].into_inner() as f64],
          [r[0].into_inner() as f64, r[1].into_inner() as f64],
        );
        if orient > 0.0 {
          Ordering::Greater
        } else if orient < 0.0 {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_vector_slope(
          &[float_to_rational(vector[0].into_inner()), float_to_rational(vector[1].into_inner())],
          &[float_to_rational(p[0].into_inner()), float_to_rational(p[1].into_inner())],
          &[float_to_rational(q[0].into_inner()), float_to_rational(q[1].into_inner())],
        )
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_perp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_perp_vector_slope(
          &[float_to_rational(vector[0].into_inner()), float_to_rational(vector[1].into_inner())],
          &[float_to_rational(p[0].into_inner()), float_to_rational(p[1].into_inner())],
          &[float_to_rational(q[0].into_inner()), float_to_rational(q[1].into_inner())],
        )
      }
    })*
  };
}

macro_rules! floating_precision {
  ( $( $ty:ty ),* ) => {
    $(
      impl TotalOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Ordering {
          <$ty>::total_cmp(self, other)
        }
      }

      impl PolygonScalar for $ty {
      fn from_constant(val: i8) -> Self {
        <$ty>::from_i8(val).unwrap()
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_dist(
          &[float_to_rational(p[0]), float_to_rational(p[1])],
          &[float_to_rational(q[0]), float_to_rational(q[1])],
          &[float_to_rational(r[0]), float_to_rational(r[1])],
        )
      }

      // This function uses the arbitrary precision machinery of `geometry_predicates` to
      // quickly compute the orientation of three 2D points. This is about 10x-50x slower
      // than the inexact version.
      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        let orient = geometry_predicates::predicates::orient2d(
          [p[0] as f64, p[1] as f64],
          [q[0] as f64, q[1] as f64],
          [r[0] as f64, r[1] as f64],
        );
        if orient > 0.0 {
          Ordering::Greater
        } else if orient < 0.0 {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_vector_slope(
          &[float_to_rational(vector[0]), float_to_rational(vector[1])],
          &[float_to_rational(p[0]), float_to_rational(p[1])],
          &[float_to_rational(q[0]), float_to_rational(q[1])],
        )
      }
      // FIXME: Use `geometry_predicates` to speed up calculation. Right now we're
      // roughly 100x slower than necessary.
      fn cmp_perp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        PolygonScalar::cmp_perp_vector_slope(
          &[float_to_rational(vector[0]), float_to_rational(vector[1])],
          &[float_to_rational(p[0]), float_to_rational(p[1])],
          &[float_to_rational(q[0]), float_to_rational(q[1])],
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
wrapped_floating_precision!(ordered_float::OrderedFloat<f32>);
wrapped_floating_precision!(ordered_float::OrderedFloat<f64>);
wrapped_floating_precision!(ordered_float::NotNan<f32>);
wrapped_floating_precision!(ordered_float::NotNan<f64>);
floating_precision!(f32);
floating_precision!(f64);

#[cfg(feature = "rug")]
impl TotalOrd for rug::Integer {
  fn total_cmp(&self, other: &Self) -> Ordering {
    self.cmp(other)
  }
}

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

fn float_to_rational(f: impl num::traits::float::FloatCore) -> num::BigRational {
  num::BigRational::from_float(f).expect("cannot convert NaN or infinite to exact precision number")
}

#[cfg(test)]
pub mod testing;
