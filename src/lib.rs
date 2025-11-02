// #![deny(warnings)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::lossy_float_literal)]
#![doc(test(no_crate_inject))]
#![doc(html_playground_url = "https://rgeometry.org/playground.html")]
#![doc(test(no_crate_inject))]
use num_bigint::BigInt;
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
  fn approx_intersection_point(
    p1: [Self; 2],
    p2: [Self; 2],
    q1: [Self; 2],
    q2: [Self; 2],
  ) -> Option<[Self; 2]> {
    approx_intersection_point_basic(Self::from_constant(0), p1, p2, q1, q2)
  }
  fn incircle(a: &[Self; 2], b: &[Self; 2], c: &[Self; 2], d: &[Self; 2]) -> Ordering;
}

fn approx_intersection_point_basic<T: Clone + NumOps<T, T> + PartialEq + std::fmt::Debug>(
  zero: T,
  p1: [T; 2],
  p2: [T; 2],
  q1: [T; 2],
  q2: [T; 2],
) -> Option<[T; 2]> {
  let [x1, y1] = p1;
  let [x2, y2] = p2;
  let [x3, y3] = q1;
  let [x4, y4] = q2;
  let denom: T = (x1.clone() - x2.clone()) * (y3.clone() - y4.clone())
    - (y1.clone() - y2.clone()) * (x3.clone() - x4.clone());
  if denom == zero {
    return None;
  }
  let part_a = x1.clone() * y2.clone() - y1.clone() * x2.clone();
  let part_b = x3.clone() * y4.clone() - y3.clone() * x4.clone();
  let x_num =
    part_a.clone() * (x3.clone() - x4.clone()) - (x1.clone() - x2.clone()) * part_b.clone();
  let y_num = part_a * (y3.clone() - y4.clone()) - (y1.clone() - y2.clone()) * part_b;
  Some([x_num / denom.clone(), y_num / denom])
}

fn incircle_exact<T>(a: &[T; 2], b: &[T; 2], c: &[T; 2], d: &[T; 2]) -> Ordering
where
  T: Clone + NumOps<T, T> + Zero + Ord,
{
  let ax = a[0].clone() - d[0].clone();
  let ay = a[1].clone() - d[1].clone();
  let bx = b[0].clone() - d[0].clone();
  let by = b[1].clone() - d[1].clone();
  let cx = c[0].clone() - d[0].clone();
  let cy = c[1].clone() - d[1].clone();

  let a_len = ax.clone() * ax.clone() + ay.clone() * ay.clone();
  let b_len = bx.clone() * bx.clone() + by.clone() * by.clone();
  let c_len = cx.clone() * cx.clone() + cy.clone() * cy.clone();

  let det = det3_generic(&ax, &ay, &a_len, &bx, &by, &b_len, &cx, &cy, &c_len);
  det.cmp(&T::zero())
}

#[allow(clippy::too_many_arguments)]
fn det3_generic<T>(a1: &T, a2: &T, a3: &T, b1: &T, b2: &T, b3: &T, c1: &T, c2: &T, c3: &T) -> T
where
  T: Clone + NumOps<T, T>,
{
  let m1 = b2.clone() * c3.clone() - b3.clone() * c2.clone();
  let m2 = b1.clone() * c3.clone() - b3.clone() * c1.clone();
  let m3 = b1.clone() * c2.clone() - b2.clone() * c1.clone();
  a1.clone() * m1 - a2.clone() * m2 + a3.clone() * m3
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
      fn approx_intersection_point(
        p1: [Self; 2],
        p2: [Self; 2],
        q1: [Self; 2],
        q2: [Self; 2],
      ) -> Option<[Self; 2]> {
        let long_p1: [$long; 2] = [p1[0] as $long, p1[1] as $long];
        let long_p2: [$long; 2] = [p2[0] as $long, p2[1] as $long];
        let long_q1: [$long; 2] = [q1[0] as $long, q1[1] as $long];
        let long_q2: [$long; 2] = [q2[0] as $long, q2[1] as $long];
        let long_result = approx_intersection_point_basic(0, long_p1, long_p2, long_q1, long_q2);
        long_result.and_then(|[x, y]| Some([Self::try_from(x).ok()?, Self::try_from(y).ok()?]))
      }
      fn incircle(a: &[Self; 2], b: &[Self; 2], c: &[Self; 2], d: &[Self; 2]) -> Ordering {
        fn to_rational(val: $ty) -> num_rational::BigRational {
          num_rational::BigRational::from_integer(BigInt::from(val))
        }
        let a_r = [to_rational(a[0]), to_rational(a[1])];
        let b_r = [to_rational(b[0]), to_rational(b[1])];
        let c_r = [to_rational(c[0]), to_rational(c[1])];
        let d_r = [to_rational(d[0]), to_rational(d[1])];
        incircle_exact(&a_r, &b_r, &c_r, &d_r)
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
      fn incircle(
        a: &[Self; 2],
        b: &[Self; 2],
        c: &[Self; 2],
        d: &[Self; 2],
      ) -> Ordering {
        incircle_exact(a, b, c, d)
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
      fn incircle(
        a: &[Self; 2],
        b: &[Self; 2],
        c: &[Self; 2],
        d: &[Self; 2],
      ) -> Ordering {
        let incircle = geometry_predicates::predicates::incircle(
          [a[0].into_inner() as f64, a[1].into_inner() as f64],
          [b[0].into_inner() as f64, b[1].into_inner() as f64],
          [c[0].into_inner() as f64, c[1].into_inner() as f64],
          [d[0].into_inner() as f64, d[1].into_inner() as f64],
        );
        if incircle > 0.0 {
          Ordering::Greater
        } else if incircle < 0.0 {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      }
    })*
  };
}

macro_rules! floating_precision {
  ( $( $ty:ty ),* ) => {
    $(
      impl TotalOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Ordering {
          // The ordering established by `<$ty>::total_cmp` does not always
          // agree with the PartialOrd and PartialEq implementations of <$ty>. For
          // example, they consider negative and positive zero equal, while
          // total_cmp doesn’t.
          if *self == 0.0 && *other == 0.0 {
            Ordering::Equal
          } else {
            <$ty>::total_cmp(self, other)
          }
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
      fn incircle(
        a: &[Self; 2],
        b: &[Self; 2],
        c: &[Self; 2],
        d: &[Self; 2],
      ) -> Ordering {
        let incircle = geometry_predicates::predicates::incircle(
          [a[0] as f64, a[1] as f64],
          [b[0] as f64, b[1] as f64],
          [c[0] as f64, c[1] as f64],
          [d[0] as f64, d[1] as f64],
        );
        if incircle > 0.0 {
          Ordering::Greater
        } else if incircle < 0.0 {
          Ordering::Less
        } else {
          Ordering::Equal
        }
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
  fn incircle(a: &[Self; 2], b: &[Self; 2], c: &[Self; 2], d: &[Self; 2]) -> Ordering {
    let mut ax = a[0].clone();
    ax -= &d[0];
    let mut ay = a[1].clone();
    ay -= &d[1];
    let mut bx = b[0].clone();
    bx -= &d[0];
    let mut by = b[1].clone();
    by -= &d[1];
    let mut cx = c[0].clone();
    cx -= &d[0];
    let mut cy = c[1].clone();
    cy -= &d[1];

    let a_len = ax.clone() * ax.clone() + ay.clone() * ay.clone();
    let b_len = bx.clone() * bx.clone() + by.clone() * by.clone();
    let c_len = cx.clone() * cx.clone() + cy.clone() * cy.clone();

    let det = det3_generic(&ax, &ay, &a_len, &bx, &by, &b_len, &cx, &cy, &c_len);
    det.cmp(&rug::Integer::new())
  }
}

fn float_to_rational(f: impl num::traits::float::FloatCore) -> num::BigRational {
  num::BigRational::from_float(f).expect("cannot convert NaN or infinite to exact precision number")
}

#[cfg(test)]
pub mod testing;
