// #![deny(warnings)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::lossy_float_literal)]
#![doc(test(no_crate_inject))]
#![doc(html_playground_url = "https://rgeometry.org/playground.html")]
#![doc(test(no_crate_inject))]
//! # Computational Geometry
//!
//! RGeometry provides data types (points, polygons, lines, segments) and algorithms for
//! computational geometry. Computational geometry is the study of algorithms for solving
//! geometric problems, such as computing convex hulls, triangulating polygons, finding
//! intersections, and determining visibility.
//!
//! This crate includes algorithms for:
//! - **Convex hulls**: Graham scan, gift wrapping, and Melkman's algorithm
//! - **Triangulation**: Ear clipping for simple polygons and constrained Delaunay triangulation
//! - **Polygonization**: Converting point sets to polygons (monotone, star-shaped, two-opt)
//! - **Line segment intersection**: Naive and Bentley-Ottmann sweep line algorithms
//! - **Visibility**: Computing visibility polygons
//! - **Spatial indexing**: Z-order curve hashing
//!
//! # Demos
//!
//! Interactive visualizations are available:
//!
//! - [Convex Hull](../../../convex_hull.html) - Graham scan algorithm
//! - [Melkman's Algorithm](../../../melkman.html) - Online convex hull for simple polylines
//! - [Ear Clipping](../../../earclip.html) - Polygon triangulation
//! - [Constrained Delaunay](../../../delaunay.html) - Ear-clipping plus Lawson flips
//! - [Two-Opt](../../../two_opt.html) - Polygonization heuristic
//! - [Random Convex](../../../random_convex.html) - Convex polygon generation
//! - [Random Monotone](../../../random_monotone.html) - Monotone polygon generation
//!
//! # Caveats
//!
//! RGeometry supports multiple numeric types, each with different trade-offs:
//!
//! - **Arbitrary precision** (e.g., `num::BigRational`): Always produces mathematically correct
//!   results but can be significantly slower due to dynamic memory allocation and arbitrary
//!   precision arithmetic.
//!
//! - **Fixed precision integers** (e.g., `i32`, `i64`): RGeometry guarantees that intermediate
//!   calculations will not overflow or underflow, but final outputs may not be representable in
//!   the chosen type. For example, testing if two line segments overlap is always safe, but
//!   computing where they intersect may produce a point whose coordinates cannot be represented.
//!
//! - **Floating point** (e.g., `f32`, `f64`): RGeometry uses robust geometric predicates to
//!   ensure accumulated rounding errors do not affect algorithmic results. However, individual
//!   coordinate values are still subject to floating-point precision limits.

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

  /// Compute orientation using an edge's outward normal as the direction vector.
  ///
  /// This is equivalent to `cmp_vector_slope(&[edge_end[1] - edge_start[1], edge_start[0] - edge_end[0]], p, q)`
  /// but avoids overflow that would occur from computing the normal explicitly.
  ///
  /// The outward normal of edge (edge_start → edge_end) for a CCW polygon is the edge vector
  /// rotated 90° clockwise: (dy, -dx) where (dx, dy) = edge_end - edge_start.
  fn cmp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering;

  /// Compute orientation using the perpendicular of an edge's outward normal.
  ///
  /// This is equivalent to `cmp_perp_vector_slope(&[edge_end[1] - edge_start[1], edge_start[0] - edge_end[0]], p, q)`
  /// but avoids overflow.
  fn cmp_perp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering;

  /// Compare the cross product of two edge normals.
  ///
  /// Returns the sign of: normal1 × normal2
  /// where normal1 = outward normal of (ref_edge_start → ref_edge_end)
  /// and normal2 = outward normal of (edge_start → edge_end)
  ///
  /// This is mathematically equivalent to the cross product of the edge vectors,
  /// avoiding the need to compute the normals explicitly.
  fn cmp_edge_normals_cross(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering;

  /// Compare the dot product of two edge normals against zero.
  ///
  /// Returns the sign of: normal1 · normal2
  /// where normal1 = outward normal of (ref_edge_start → ref_edge_end)
  /// and normal2 = outward normal of (edge_start → edge_end)
  ///
  /// This is mathematically equivalent to the dot product of the edge vectors,
  /// avoiding the need to compute the normals explicitly.
  fn cmp_edge_normals_dot(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering;

  /// Compare the cross product of an edge's outward normal with a direction vector.
  ///
  /// Returns the sign of: edge_normal × direction
  /// This equals the dot product of (edge_end - edge_start) · direction.
  fn cmp_edge_normal_cross_direction(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    direction: &[Self; 2],
  ) -> std::cmp::Ordering;

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
  ( $ty:ty, $uty:ty, $long:ty, $ulong: ty, $apfp_mod:path ) => {
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
        use $apfp_mod as apfp_mod;
        apfp_mod::cmp_dist(
          &apfp_mod::Coord::new(p[0], p[1]),
          &apfp_mod::Coord::new(q[0], q[1]),
          &apfp_mod::Coord::new(r[0], r[1]),
        )
      }

      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        use apfp::geometry::Orientation;
        use $apfp_mod as apfp_mod;
        let orient = apfp_mod::orient2d(
          &apfp_mod::Coord::new(p[0], p[1]),
          &apfp_mod::Coord::new(q[0], q[1]),
          &apfp_mod::Coord::new(r[0], r[1]),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }

      fn cmp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        use apfp::geometry::Orientation;
        use $apfp_mod as apfp_mod;
        let orient = apfp_mod::orient2d_vec(
          &apfp_mod::Coord::new(p[0], p[1]),
          &apfp_mod::Coord::new(vector[0], vector[1]),
          &apfp_mod::Coord::new(q[0], q[1]),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }

      fn cmp_perp_vector_slope(
        vector: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        use apfp::geometry::Orientation;
        use $apfp_mod as apfp_mod;
        let orient = apfp_mod::orient2d_normal(
          &apfp_mod::Coord::new(p[0], p[1]),
          &apfp_mod::Coord::new(vector[0], vector[1]),
          &apfp_mod::Coord::new(q[0], q[1]),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }

      fn cmp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // We want: cmp_vector_slope(edge_normal, p, q)
        // Which computes orient2d_vec(p, edge_normal, q):
        //   sign((p[0] - q[0]) * edge_normal[1] - (p[1] - q[1]) * edge_normal[0])
        //   = sign((p[0] - q[0]) * (edge_start[0] - edge_end[0]) - (p[1] - q[1]) * (edge_end[1] - edge_start[1]))
        let e0 = edge_start[0];
        let e1 = edge_start[1];
        let f0 = edge_end[0];
        let f1 = edge_end[1];
        let p0 = p[0];
        let p1 = p[1];
        let q0 = q[0];
        let q1 = q[1];
        let sign = apfp::int_signum!((p0 - q0) * (e0 - f0) - (p1 - q1) * (f1 - e1));
        match sign {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          0 => Ordering::Equal,
          _ => unreachable!(),
        }
      }

      fn cmp_perp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // We want: cmp_perp_vector_slope(edge_normal, p, q)
        // Which computes orient2d_normal(p, edge_normal, q):
        //   sign((p[0] - q[0]) * edge_normal[0] + (p[1] - q[1]) * edge_normal[1])
        //   = sign((p[0] - q[0]) * (edge_end[1] - edge_start[1]) + (p[1] - q[1]) * (edge_start[0] - edge_end[0]))
        let e0 = edge_start[0];
        let e1 = edge_start[1];
        let f0 = edge_end[0];
        let f1 = edge_end[1];
        let p0 = p[0];
        let p1 = p[1];
        let q0 = q[0];
        let q1 = q[1];
        let sign = apfp::int_signum!((p0 - q0) * (f1 - e1) + (p1 - q1) * (e0 - f0));
        match sign {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          0 => Ordering::Equal,
          _ => unreachable!(),
        }
      }

      fn cmp_edge_normals_cross(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // ref_normal = (ref_edge_end[1] - ref_edge_start[1], ref_edge_start[0] - ref_edge_end[0])
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // ref_normal × edge_normal = ref_normal.x * edge_normal.y - ref_normal.y * edge_normal.x
        // = (ref_edge_end[1] - ref_edge_start[1]) * (edge_start[0] - edge_end[0])
        //   - (ref_edge_start[0] - ref_edge_end[0]) * (edge_end[1] - edge_start[1])
        let re0 = ref_edge_start[0];
        let re1 = ref_edge_start[1];
        let rf0 = ref_edge_end[0];
        let rf1 = ref_edge_end[1];
        let e0 = edge_start[0];
        let e1 = edge_start[1];
        let f0 = edge_end[0];
        let f1 = edge_end[1];
        let sign = apfp::int_signum!((rf1 - re1) * (e0 - f0) - (re0 - rf0) * (f1 - e1));
        match sign {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          0 => Ordering::Equal,
          _ => unreachable!(),
        }
      }

      fn cmp_edge_normals_dot(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // ref_normal = (ref_edge_end[1] - ref_edge_start[1], ref_edge_start[0] - ref_edge_end[0])
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // ref_normal · edge_normal = ref_normal.x * edge_normal.x + ref_normal.y * edge_normal.y
        // = (rf1 - re1) * (f1 - e1) + (re0 - rf0) * (e0 - f0)
        // This equals the dot product of the edge vectors: (rf0 - re0) * (f0 - e0) + (rf1 - re1) * (f1 - e1)
        let re0 = ref_edge_start[0];
        let re1 = ref_edge_start[1];
        let rf0 = ref_edge_end[0];
        let rf1 = ref_edge_end[1];
        let e0 = edge_start[0];
        let e1 = edge_start[1];
        let f0 = edge_end[0];
        let f1 = edge_end[1];
        let sign = apfp::int_signum!((rf0 - re0) * (f0 - e0) + (rf1 - re1) * (f1 - e1));
        match sign {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          0 => Ordering::Equal,
          _ => unreachable!(),
        }
      }

      fn cmp_edge_normal_cross_direction(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        direction: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // edge_normal × direction = edge_normal.x * direction[1] - edge_normal.y * direction[0]
        // = (edge_end[1] - edge_start[1]) * direction[1] - (edge_start[0] - edge_end[0]) * direction[0]
        // = (edge_end[1] - edge_start[1]) * direction[1] + (edge_end[0] - edge_start[0]) * direction[0]
        let e0 = edge_start[0];
        let e1 = edge_start[1];
        let f0 = edge_end[0];
        let f1 = edge_end[1];
        let d0 = direction[0];
        let d1 = direction[1];
        let sign = apfp::int_signum!((f1 - e1) * d1 + (f0 - e0) * d0);
        match sign {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          0 => Ordering::Equal,
          _ => unreachable!(),
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
        use apfp::geometry::Orientation;
        use $apfp_mod as apfp_mod;
        let result = apfp_mod::incircle(
          &apfp_mod::Coord::new(a[0], a[1]),
          &apfp_mod::Coord::new(b[0], b[1]),
          &apfp_mod::Coord::new(c[0], c[1]),
          &apfp_mod::Coord::new(d[0], d[1]),
        );
        match result {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
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
      fn cmp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        let edge_normal = [
          &edge_end[1] - &edge_start[1],
          &edge_start[0] - &edge_end[0],
        ];
        PolygonScalar::cmp_vector_slope(&edge_normal, p, q)
      }
      fn cmp_perp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        let edge_normal = [
          &edge_end[1] - &edge_start[1],
          &edge_start[0] - &edge_end[0],
        ];
        PolygonScalar::cmp_perp_vector_slope(&edge_normal, p, q)
      }
      fn cmp_edge_normals_cross(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // ref_normal × edge_normal
        let ref_nx = &ref_edge_end[1] - &ref_edge_start[1];
        let ref_ny = &ref_edge_start[0] - &ref_edge_end[0];
        let edge_nx = &edge_end[1] - &edge_start[1];
        let edge_ny = &edge_start[0] - &edge_end[0];
        let cross = &ref_nx * &edge_ny - &ref_ny * &edge_nx;
        let zero = Self::from_constant(0);
        cross.cmp(&zero)
      }
      fn cmp_edge_normals_dot(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // ref_normal · edge_normal = ref_edge_vector · edge_vector
        let dot = (&ref_edge_end[0] - &ref_edge_start[0]) * (&edge_end[0] - &edge_start[0])
          + (&ref_edge_end[1] - &ref_edge_start[1]) * (&edge_end[1] - &edge_start[1]);
        let zero = Self::from_constant(0);
        dot.cmp(&zero)
      }
      fn cmp_edge_normal_cross_direction(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        direction: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal × direction = (edge_end - edge_start) · direction
        let dot = (&edge_end[0] - &edge_start[0]) * &direction[0]
          + (&edge_end[1] - &edge_start[1]) * &direction[1];
        let zero = Self::from_constant(0);
        dot.cmp(&zero)
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

      // This function uses the arbitrary precision machinery of `apfp` to
      // quickly compute the orientation of three 2D points.
      fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
        use apfp::geometry::{Orientation, f64::Coord};
        let orient = apfp::geometry::f64::orient2d(
          &Coord::new(p[0] as f64, p[1] as f64),
          &Coord::new(q[0] as f64, q[1] as f64),
          &Coord::new(r[0] as f64, r[1] as f64),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }
      // Uses apfp's orient2d_vec for fast arbitrary precision calculation.
      fn cmp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        use apfp::geometry::{Orientation, f64::Coord};
        let orient = apfp::geometry::f64::orient2d_vec(
          &Coord::new(p[0] as f64, p[1] as f64),
          &Coord::new(vector[0] as f64, vector[1] as f64),
          &Coord::new(q[0] as f64, q[1] as f64),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }
      // Uses apfp's orient2d_normal for fast arbitrary precision calculation.
      fn cmp_perp_vector_slope(vector: &[Self;2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
        use apfp::geometry::{Orientation, f64::Coord};
        let orient = apfp::geometry::f64::orient2d_normal(
          &Coord::new(p[0] as f64, p[1] as f64),
          &Coord::new(vector[0] as f64, vector[1] as f64),
          &Coord::new(q[0] as f64, q[1] as f64),
        );
        match orient {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }
      fn cmp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // We need to compute orient2d_vec(p, edge_normal, q), which is:
        //   (p.x - q.x) * normal.y - (p.y - q.y) * normal.x
        // = (p.x - q.x) * (edge_start[0] - edge_end[0]) - (p.y - q.y) * (edge_end[1] - edge_start[1])
        //
        // Use apfp_signum! with all original coordinates to avoid precision loss
        // from pre-computing the normal.
        let px = p[0] as f64;
        let py = p[1] as f64;
        let qx = q[0] as f64;
        let qy = q[1] as f64;
        let es0 = edge_start[0] as f64;
        let es1 = edge_start[1] as f64;
        let ee0 = edge_end[0] as f64;
        let ee1 = edge_end[1] as f64;
        // (p.x - q.x) * (es0 - ee0) - (p.y - q.y) * (ee1 - es1)
        // = (px - qx) * (es0 - ee0) - (py - qy) * (ee1 - es1)
        // = px*es0 - px*ee0 - qx*es0 + qx*ee0 - py*ee1 + py*es1 + qy*ee1 - qy*es1
        match apfp::apfp_signum!(
          (px - qx) * (es0 - ee0) - (py - qy) * (ee1 - es1)
        ) {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          _ => Ordering::Equal,
        }
      }
      fn cmp_perp_edge_normal_slope(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        p: &[Self; 2],
        q: &[Self; 2],
      ) -> std::cmp::Ordering {
        // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        // We need to compute orient2d_normal(p, edge_normal, q), which is:
        //   (p.x - q.x) * normal.x + (p.y - q.y) * normal.y
        // = (p.x - q.x) * (edge_end[1] - edge_start[1]) + (p.y - q.y) * (edge_start[0] - edge_end[0])
        //
        // Use apfp_signum! with all original coordinates to avoid precision loss.
        let px = p[0] as f64;
        let py = p[1] as f64;
        let qx = q[0] as f64;
        let qy = q[1] as f64;
        let es0 = edge_start[0] as f64;
        let es1 = edge_start[1] as f64;
        let ee0 = edge_end[0] as f64;
        let ee1 = edge_end[1] as f64;
        // (px - qx) * (ee1 - es1) + (py - qy) * (es0 - ee0)
        match apfp::apfp_signum!(
          (px - qx) * (ee1 - es1) + (py - qy) * (es0 - ee0)
        ) {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          _ => Ordering::Equal,
        }
      }
      fn cmp_edge_normals_cross(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // Use apfp's adaptive precision for robust computation.
        // cross = ref_nx * edge_ny - ref_ny * edge_nx
        // where ref_n = (ref_end[1] - ref_start[1], ref_start[0] - ref_end[0])
        //       edge_n = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
        let ref_nx = (ref_edge_end[1] - ref_edge_start[1]) as f64;
        let ref_ny = (ref_edge_start[0] - ref_edge_end[0]) as f64;
        let edge_nx = (edge_end[1] - edge_start[1]) as f64;
        let edge_ny = (edge_start[0] - edge_end[0]) as f64;
        match apfp::apfp_signum!(ref_nx * edge_ny - ref_ny * edge_nx) {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          _ => Ordering::Equal,
        }
      }
      fn cmp_edge_normals_dot(
        ref_edge_start: &[Self; 2],
        ref_edge_end: &[Self; 2],
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
      ) -> std::cmp::Ordering {
        // Use apfp's adaptive precision for robust computation.
        // dot = ref_vec · edge_vec
        let ref_dx = (ref_edge_end[0] - ref_edge_start[0]) as f64;
        let ref_dy = (ref_edge_end[1] - ref_edge_start[1]) as f64;
        let edge_dx = (edge_end[0] - edge_start[0]) as f64;
        let edge_dy = (edge_end[1] - edge_start[1]) as f64;
        match apfp::apfp_signum!(ref_dx * edge_dx + ref_dy * edge_dy) {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          _ => Ordering::Equal,
        }
      }
      fn cmp_edge_normal_cross_direction(
        edge_start: &[Self; 2],
        edge_end: &[Self; 2],
        direction: &[Self; 2],
      ) -> std::cmp::Ordering {
        // Use apfp's adaptive precision for robust computation.
        // edge_normal × direction = edge_vec · direction
        let edge_dx = (edge_end[0] - edge_start[0]) as f64;
        let edge_dy = (edge_end[1] - edge_start[1]) as f64;
        let dir_x = direction[0] as f64;
        let dir_y = direction[1] as f64;
        match apfp::apfp_signum!(edge_dx * dir_x + edge_dy * dir_y) {
          1 => Ordering::Greater,
          -1 => Ordering::Less,
          _ => Ordering::Equal,
        }
      }
      fn incircle(
        a: &[Self; 2],
        b: &[Self; 2],
        c: &[Self; 2],
        d: &[Self; 2],
      ) -> Ordering {
        use apfp::geometry::{Orientation, f64::Coord};
        let result = apfp::geometry::f64::incircle(
          &Coord::new(a[0] as f64, a[1] as f64),
          &Coord::new(b[0] as f64, b[1] as f64),
          &Coord::new(c[0] as f64, c[1] as f64),
          &Coord::new(d[0] as f64, d[1] as f64),
        );
        match result {
          Orientation::CounterClockwise => Ordering::Greater,
          Orientation::Clockwise => Ordering::Less,
          Orientation::CoLinear => Ordering::Equal,
        }
      }
    })*
  };
}

fixed_precision!(i8, u8, i16, u16, apfp::geometry::i8);
fixed_precision!(i16, u16, i32, u32, apfp::geometry::i16);
fixed_precision!(i32, u32, i64, u64, apfp::geometry::i32);
fixed_precision!(i64, u64, i128, u128, apfp::geometry::i64);

// isize needs special handling since apfp doesn't have an isize module
// We cast to i64 which works on all platforms (isize is at most 64-bit)
impl TotalOrd for isize {
  fn total_cmp(&self, other: &Self) -> Ordering {
    self.cmp(other)
  }
}

impl PolygonScalar for isize {
  fn from_constant(val: i8) -> Self {
    val as isize
  }
  fn cmp_dist(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
    use apfp::geometry::i64 as apfp_mod;
    apfp_mod::cmp_dist(
      &apfp_mod::Coord::new(p[0] as i64, p[1] as i64),
      &apfp_mod::Coord::new(q[0] as i64, q[1] as i64),
      &apfp_mod::Coord::new(r[0] as i64, r[1] as i64),
    )
  }

  fn cmp_slope(p: &[Self; 2], q: &[Self; 2], r: &[Self; 2]) -> std::cmp::Ordering {
    use apfp::geometry::{Orientation, i64 as apfp_mod};
    let orient = apfp_mod::orient2d(
      &apfp_mod::Coord::new(p[0] as i64, p[1] as i64),
      &apfp_mod::Coord::new(q[0] as i64, q[1] as i64),
      &apfp_mod::Coord::new(r[0] as i64, r[1] as i64),
    );
    match orient {
      Orientation::CounterClockwise => Ordering::Greater,
      Orientation::Clockwise => Ordering::Less,
      Orientation::CoLinear => Ordering::Equal,
    }
  }

  fn cmp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
    use apfp::geometry::{Orientation, i64 as apfp_mod};
    let orient = apfp_mod::orient2d_vec(
      &apfp_mod::Coord::new(p[0] as i64, p[1] as i64),
      &apfp_mod::Coord::new(vector[0] as i64, vector[1] as i64),
      &apfp_mod::Coord::new(q[0] as i64, q[1] as i64),
    );
    match orient {
      Orientation::CounterClockwise => Ordering::Greater,
      Orientation::Clockwise => Ordering::Less,
      Orientation::CoLinear => Ordering::Equal,
    }
  }

  fn cmp_perp_vector_slope(vector: &[Self; 2], p: &[Self; 2], q: &[Self; 2]) -> std::cmp::Ordering {
    use apfp::geometry::{Orientation, i64 as apfp_mod};
    let orient = apfp_mod::orient2d_normal(
      &apfp_mod::Coord::new(p[0] as i64, p[1] as i64),
      &apfp_mod::Coord::new(vector[0] as i64, vector[1] as i64),
      &apfp_mod::Coord::new(q[0] as i64, q[1] as i64),
    );
    match orient {
      Orientation::CounterClockwise => Ordering::Greater,
      Orientation::Clockwise => Ordering::Less,
      Orientation::CoLinear => Ordering::Equal,
    }
  }

  fn cmp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering {
    // Cast to i64 and use int_signum for overflow-safe computation
    let (e0, e1) = (edge_start[0] as i64, edge_start[1] as i64);
    let (f0, f1) = (edge_end[0] as i64, edge_end[1] as i64);
    let (p0, p1) = (p[0] as i64, p[1] as i64);
    let (q0, q1) = (q[0] as i64, q[1] as i64);
    let sign = apfp::int_signum!((p0 - q0) * (e0 - f0) - (p1 - q1) * (f1 - e1));
    match sign {
      1 => Ordering::Greater,
      -1 => Ordering::Less,
      0 => Ordering::Equal,
      _ => unreachable!(),
    }
  }

  fn cmp_perp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering {
    // Cast to i64 and use int_signum for overflow-safe computation
    let (e0, e1) = (edge_start[0] as i64, edge_start[1] as i64);
    let (f0, f1) = (edge_end[0] as i64, edge_end[1] as i64);
    let (p0, p1) = (p[0] as i64, p[1] as i64);
    let (q0, q1) = (q[0] as i64, q[1] as i64);
    let sign = apfp::int_signum!((p0 - q0) * (f1 - e1) + (p1 - q1) * (e0 - f0));
    match sign {
      1 => Ordering::Greater,
      -1 => Ordering::Less,
      0 => Ordering::Equal,
      _ => unreachable!(),
    }
  }

  fn cmp_edge_normals_cross(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering {
    let (re0, re1) = (ref_edge_start[0] as i64, ref_edge_start[1] as i64);
    let (rf0, rf1) = (ref_edge_end[0] as i64, ref_edge_end[1] as i64);
    let (e0, e1) = (edge_start[0] as i64, edge_start[1] as i64);
    let (f0, f1) = (edge_end[0] as i64, edge_end[1] as i64);
    let sign = apfp::int_signum!((rf1 - re1) * (e0 - f0) - (re0 - rf0) * (f1 - e1));
    match sign {
      1 => Ordering::Greater,
      -1 => Ordering::Less,
      0 => Ordering::Equal,
      _ => unreachable!(),
    }
  }

  fn cmp_edge_normals_dot(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering {
    let (re0, re1) = (ref_edge_start[0] as i64, ref_edge_start[1] as i64);
    let (rf0, rf1) = (ref_edge_end[0] as i64, ref_edge_end[1] as i64);
    let (e0, e1) = (edge_start[0] as i64, edge_start[1] as i64);
    let (f0, f1) = (edge_end[0] as i64, edge_end[1] as i64);
    let sign = apfp::int_signum!((rf0 - re0) * (f0 - e0) + (rf1 - re1) * (f1 - e1));
    match sign {
      1 => Ordering::Greater,
      -1 => Ordering::Less,
      0 => Ordering::Equal,
      _ => unreachable!(),
    }
  }

  fn cmp_edge_normal_cross_direction(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    direction: &[Self; 2],
  ) -> std::cmp::Ordering {
    let (e0, e1) = (edge_start[0] as i64, edge_start[1] as i64);
    let (f0, f1) = (edge_end[0] as i64, edge_end[1] as i64);
    let (d0, d1) = (direction[0] as i64, direction[1] as i64);
    let sign = apfp::int_signum!((f1 - e1) * d1 + (f0 - e0) * d0);
    match sign {
      1 => Ordering::Greater,
      -1 => Ordering::Less,
      0 => Ordering::Equal,
      _ => unreachable!(),
    }
  }
  fn approx_intersection_point(
    p1: [Self; 2],
    p2: [Self; 2],
    q1: [Self; 2],
    q2: [Self; 2],
  ) -> Option<[Self; 2]> {
    let long_p1: [i128; 2] = [p1[0] as i128, p1[1] as i128];
    let long_p2: [i128; 2] = [p2[0] as i128, p2[1] as i128];
    let long_q1: [i128; 2] = [q1[0] as i128, q1[1] as i128];
    let long_q2: [i128; 2] = [q2[0] as i128, q2[1] as i128];
    let long_result = approx_intersection_point_basic(0, long_p1, long_p2, long_q1, long_q2);
    long_result.and_then(|[x, y]| Some([Self::try_from(x).ok()?, Self::try_from(y).ok()?]))
  }
  fn incircle(a: &[Self; 2], b: &[Self; 2], c: &[Self; 2], d: &[Self; 2]) -> Ordering {
    use apfp::geometry::{Orientation, i64 as apfp_mod};
    let result = apfp_mod::incircle(
      &apfp_mod::Coord::new(a[0] as i64, a[1] as i64),
      &apfp_mod::Coord::new(b[0] as i64, b[1] as i64),
      &apfp_mod::Coord::new(c[0] as i64, c[1] as i64),
      &apfp_mod::Coord::new(d[0] as i64, d[1] as i64),
    );
    match result {
      Orientation::CounterClockwise => Ordering::Greater,
      Orientation::Clockwise => Ordering::Less,
      Orientation::CoLinear => Ordering::Equal,
    }
  }
}
arbitrary_precision!(num_bigint::BigInt);
arbitrary_precision!(num_rational::BigRational);
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
    // The perpendicular (normal) of vector (vx, vy) rotated 90° counterclockwise is (-vy, vx)
    // So the new point is: (p[0] - vector[1], p[1] + vector[0])
    let new_x = rug::Integer::from(&p[0] - &vector[1]);
    let new_y = rug::Integer::from(&p[1] + &vector[0]);
    PolygonScalar::cmp_slope(p, &[new_x, new_y], q)
  }
  fn cmp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering {
    // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
    let edge_normal = [
      rug::Integer::from(&edge_end[1] - &edge_start[1]),
      rug::Integer::from(&edge_start[0] - &edge_end[0]),
    ];
    PolygonScalar::cmp_vector_slope(&edge_normal, p, q)
  }
  fn cmp_perp_edge_normal_slope(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    p: &[Self; 2],
    q: &[Self; 2],
  ) -> std::cmp::Ordering {
    // edge_normal = (edge_end[1] - edge_start[1], edge_start[0] - edge_end[0])
    let edge_normal = [
      rug::Integer::from(&edge_end[1] - &edge_start[1]),
      rug::Integer::from(&edge_start[0] - &edge_end[0]),
    ];
    PolygonScalar::cmp_perp_vector_slope(&edge_normal, p, q)
  }
  fn cmp_edge_normals_cross(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering {
    // ref_normal × edge_normal
    let ref_nx = rug::Integer::from(&ref_edge_end[1] - &ref_edge_start[1]);
    let ref_ny = rug::Integer::from(&ref_edge_start[0] - &ref_edge_end[0]);
    let edge_nx = rug::Integer::from(&edge_end[1] - &edge_start[1]);
    let edge_ny = rug::Integer::from(&edge_start[0] - &edge_end[0]);
    let cross = ref_nx * edge_ny - ref_ny * edge_nx;
    cross.cmp(&rug::Integer::new())
  }
  fn cmp_edge_normals_dot(
    ref_edge_start: &[Self; 2],
    ref_edge_end: &[Self; 2],
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
  ) -> std::cmp::Ordering {
    // ref_normal · edge_normal = ref_edge_vector · edge_vector
    let dot = rug::Integer::from(&ref_edge_end[0] - &ref_edge_start[0])
      * rug::Integer::from(&edge_end[0] - &edge_start[0])
      + rug::Integer::from(&ref_edge_end[1] - &ref_edge_start[1])
        * rug::Integer::from(&edge_end[1] - &edge_start[1]);
    dot.cmp(&rug::Integer::new())
  }
  fn cmp_edge_normal_cross_direction(
    edge_start: &[Self; 2],
    edge_end: &[Self; 2],
    direction: &[Self; 2],
  ) -> std::cmp::Ordering {
    // edge_normal × direction = (edge_end - edge_start) · direction
    let dot = rug::Integer::from(&edge_end[0] - &edge_start[0]) * &direction[0]
      + rug::Integer::from(&edge_end[1] - &edge_start[1]) * &direction[1];
    dot.cmp(&rug::Integer::new())
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

#[cfg(test)]
mod floating_robustness_tests {
  use super::*;

  // Helper to convert f64 to BigRational for ground truth comparison.
  fn to_big(f: f64) -> num_rational::BigRational {
    num_rational::BigRational::from_float(f).unwrap()
  }

  // Test that cmp_edge_normals_cross is robust for nearly-parallel edges.
  //
  // The naive floating-point implementation computes:
  //   cross = ref_nx * edge_ny - ref_ny * edge_nx
  //
  // For edges from origin, this simplifies to:
  //   cross = ref_end[0] * edge_end[1] - ref_end[1] * edge_end[0]
  //
  // At 1e15 scale, the products are at 1e30 scale where the ULP is ~1.4e14.
  // This means differences smaller than 1.4e14 get rounded away!
  //
  // This test uses coordinates where:
  // - All inputs are exactly representable f64 values
  // - The true cross product is -2 (mathematically exact, computable by BigRational)
  // - The naive f64 computation gives 0 due to precision loss in the products
  #[test]
  fn cmp_edge_normals_cross_f64_robustness() {
    // Edges from origin for simplicity:
    // ref_edge: (0,0) -> (1e15, 1e15+1)
    // edge: (0,0) -> (1e15+2, 1e15+3)
    //
    // cross = ref[0]*edge[1] - ref[1]*edge[0]
    //       = 1e15 * (1e15+3) - (1e15+1) * (1e15+2)
    //       = 1e30 + 3e15 - (1e30 + 2e15 + 1e15 + 2)
    //       = 1e30 + 3e15 - 1e30 - 3e15 - 2
    //       = -2
    //
    // But in f64:
    // - 1e15 * (1e15+3) = 1e30 + 3e15, rounded to nearest at 1e30 scale
    // - (1e15+1) * (1e15+2) = 1e30 + 3e15 + 2, rounded to same value
    // - Both products round to the same f64 value, so cross = 0
    //
    // ULP at 1e30 scale is ~1.4e14, so the difference of 2 is completely lost.
    let big = 1e15f64;

    let ref_start = [0.0f64, 0.0];
    let ref_end = [big, big + 1.0];
    let edge_start = [0.0f64, 0.0];
    let edge_end = [big + 2.0, big + 3.0];

    // Ground truth using BigRational
    let ref_start_big = [to_big(ref_start[0]), to_big(ref_start[1])];
    let ref_end_big = [to_big(ref_end[0]), to_big(ref_end[1])];
    let edge_start_big = [to_big(edge_start[0]), to_big(edge_start[1])];
    let edge_end_big = [to_big(edge_end[0]), to_big(edge_end[1])];
    let expected = num_rational::BigRational::cmp_edge_normals_cross(
      &ref_start_big,
      &ref_end_big,
      &edge_start_big,
      &edge_end_big,
    );

    // The mathematically correct answer is Less (cross = -2)
    assert_eq!(
      expected,
      Ordering::Less,
      "BigRational should give Less (cross = -2)"
    );

    // The f64 implementation should match BigRational
    let result = f64::cmp_edge_normals_cross(&ref_start, &ref_end, &edge_start, &edge_end);
    assert_eq!(
      result, expected,
      "f64 cmp_edge_normals_cross gave {:?} but BigRational gave {:?}. \
       This demonstrates precision loss: true cross is -2, but naive f64 gives 0.",
      result, expected
    );
  }

  // Test that cmp_edge_normals_dot is robust for nearly-perpendicular edges.
  //
  // The dot product of edge vectors suffers similar precision loss.
  // dot = (ref_end[0] - ref_start[0]) * (edge_end[0] - edge_start[0])
  //     + (ref_end[1] - ref_start[1]) * (edge_end[1] - edge_start[1])
  //
  // For edges from origin:
  #[test]
  fn cmp_edge_normals_dot_f64_robustness() {
    let big = 1e15f64;

    let ref_start = [0.0f64, 0.0];
    let ref_end = [big, 1.0];
    let edge_start = [0.0f64, 0.0];
    let edge_end = [-1.0, big + 0.5];

    let ref_start_big = [to_big(ref_start[0]), to_big(ref_start[1])];
    let ref_end_big = [to_big(ref_end[0]), to_big(ref_end[1])];
    let edge_start_big = [to_big(edge_start[0]), to_big(edge_start[1])];
    let edge_end_big = [to_big(edge_end[0]), to_big(edge_end[1])];
    let expected = num_rational::BigRational::cmp_edge_normals_dot(
      &ref_start_big,
      &ref_end_big,
      &edge_start_big,
      &edge_end_big,
    );

    assert_eq!(expected, Ordering::Greater);

    let result = f64::cmp_edge_normals_dot(&ref_start, &ref_end, &edge_start, &edge_end);
    assert_eq!(result, expected);
  }

  // Test that cmp_edge_normal_cross_direction is robust.
  //
  // This computes edge_vec · direction, which has the same structure as dot product.
  #[test]
  fn cmp_edge_normal_cross_direction_f64_robustness() {
    // edge: (0, 0) -> (1e15, 1)
    // direction: (-1, 1e15 + 0.5)
    //
    // edge_vec · direction = 1e15 * (-1) + 1 * (1e15 + 0.5) = 0.5
    let big = 1e15f64;

    let edge_start = [0.0f64, 0.0];
    let edge_end = [big, 1.0];
    let direction = [-1.0f64, big + 0.5];

    let edge_start_big = [to_big(edge_start[0]), to_big(edge_start[1])];
    let edge_end_big = [to_big(edge_end[0]), to_big(edge_end[1])];
    let direction_big = [to_big(direction[0]), to_big(direction[1])];
    let expected = num_rational::BigRational::cmp_edge_normal_cross_direction(
      &edge_start_big,
      &edge_end_big,
      &direction_big,
    );

    assert_eq!(
      expected,
      Ordering::Greater,
      "BigRational should give Greater (result = 0.5)"
    );

    let result = f64::cmp_edge_normal_cross_direction(&edge_start, &edge_end, &direction);
    assert_eq!(
      result, expected,
      "f64 cmp_edge_normal_cross_direction gave {:?} but BigRational gave {:?}",
      result, expected
    );
  }

  // Demonstrate that cmp_edge_normal_slope can disagree with BigRational when the
  // edge normal is computed via lossy f64 subtraction.
  #[test]
  fn cmp_edge_normal_slope_f64_mismatch() {
    // edge_start/end use decimal literals that are not exactly representable.
    // The edge normal is computed via subtraction in f64, which rounds.
    // With large p/q deltas, the rounding error changes the sign.
    let edge_start = [0.1f64, 0.2];
    let edge_end = [0.5f64, 0.1];
    let p = [1125899906842624.0f64, 4503599627370496.0];
    let q = [0.0f64, 0.0];

    let edge_start_big = [to_big(edge_start[0]), to_big(edge_start[1])];
    let edge_end_big = [to_big(edge_end[0]), to_big(edge_end[1])];
    let p_big = [to_big(p[0]), to_big(p[1])];
    let q_big = [to_big(q[0]), to_big(q[1])];

    let expected = num_rational::BigRational::cmp_edge_normal_slope(
      &edge_start_big,
      &edge_end_big,
      &p_big,
      &q_big,
    );
    assert_eq!(
      expected,
      Ordering::Greater,
      "BigRational should give Greater for this configuration"
    );

    let result = f64::cmp_edge_normal_slope(&edge_start, &edge_end, &p, &q);
    assert_eq!(
      result, expected,
      "f64 cmp_edge_normal_slope gave {:?} but BigRational gave {:?}",
      result, expected
    );
  }

  // Demonstrate that cmp_perp_edge_normal_slope can disagree with BigRational when
  // the edge normal is computed via lossy f64 subtraction.
  #[test]
  fn cmp_perp_edge_normal_slope_f64_mismatch() {
    let edge_start = [0.1f64, 0.1];
    let edge_end = [0.2f64, 0.5];
    let p = [1125899906842624.0f64, 4503599627370496.0];
    let q = [0.0f64, 0.0];

    let edge_start_big = [to_big(edge_start[0]), to_big(edge_start[1])];
    let edge_end_big = [to_big(edge_end[0]), to_big(edge_end[1])];
    let p_big = [to_big(p[0]), to_big(p[1])];
    let q_big = [to_big(q[0]), to_big(q[1])];

    let expected = num_rational::BigRational::cmp_perp_edge_normal_slope(
      &edge_start_big,
      &edge_end_big,
      &p_big,
      &q_big,
    );
    assert_eq!(
      expected,
      Ordering::Less,
      "BigRational should give Less for this configuration"
    );

    let result = f64::cmp_perp_edge_normal_slope(&edge_start, &edge_end, &p, &q);
    assert_eq!(
      result, expected,
      "f64 cmp_perp_edge_normal_slope gave {:?} but BigRational gave {:?}",
      result, expected
    );
  }
}
