use std::cmp::Ordering;

use crate::PolygonScalar;
use crate::data::Vector;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum Orientation {
  CounterClockWise,
  ClockWise,
  CoLinear,
}
use Orientation::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum SoS {
  CounterClockWise,
  ClockWise,
}

// let slope1 = (2^q - 2^r) * (p - q);
// let slope2 = (2^q - 2^p) * (r - q);

impl Orientation {
  /// Determine the direction you have to turn if you walk from `p1`
  /// to `p2` to `p3`.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  ///
  /// # Polymorphism
  ///
  /// This function works with both [Points](crate::data::Point) and [Vectors](Vector). You should prefer to
  /// use [Point::orient](crate::data::Point::orient) when possible.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use rgeometry::data::Point;
  /// # use rgeometry::Orientation;
  /// let p1 = Point::new([ 0, 0 ]);
  /// let p2 = Point::new([ 0, 1 ]); // One unit above p1.
  /// // (0,0) -> (0,1) -> (0,2) == Orientation::CoLinear
  /// assert!(Orientation::new(&p1, &p2, &Point::new([ 0, 2 ])).is_colinear());
  /// // (0,0) -> (0,1) -> (-1,2) == Orientation::CounterClockWise
  /// assert!(Orientation::new(&p1, &p2, &Point::new([ -1, 2 ])).is_ccw());
  /// // (0,0) -> (0,1) -> (1,2) == Orientation::ClockWise
  /// assert!(Orientation::new(&p1, &p2, &Point::new([ 1, 2 ])).is_cw());
  /// ```
  ///
  pub fn new<T>(p1: &[T; 2], p2: &[T; 2], p3: &[T; 2]) -> Orientation
  where
    T: PolygonScalar,
  {
    // raw_arr_turn(p, q, r)
    match T::cmp_slope(p1, p2, p3) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Locate `p2` in relation to the line determined by the point `p1` and the direction
  /// vector.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  ///
  /// This function is identical to [`Orientation::new`]`(p1, p1+v, p2)` but will never
  /// cause arithmetic overflows even if `p+v` would overflow.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use rgeometry::data::{Vector,Point};
  /// # use rgeometry::Orientation;
  /// let v = Vector([ 1, 1 ]); // Vector pointing to the top-right corner.
  /// let p1 = Point::new([ 5, 5 ]);
  /// assert!(Orientation::along_vector(&p1, &v, &Point::new([ 6, 6 ])).is_colinear());
  /// assert!(Orientation::along_vector(&p1, &v, &Point::new([ 7, 8 ])).is_ccw());
  /// assert!(Orientation::along_vector(&p1, &v, &Point::new([ 8, 7 ])).is_cw());
  /// ```
  pub fn along_vector<T>(p1: &[T; 2], vector: &Vector<T, 2>, p2: &[T; 2]) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_vector_slope(&vector.0, p1, p2) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  pub fn along_perp_vector<T>(p1: &[T; 2], vector: &Vector<T, 2>, p2: &[T; 2]) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_perp_vector_slope(&vector.0, p1, p2) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Locate `p2` in relation to the line defined by the outward normal of an edge.
  ///
  /// The edge goes from `edge_start` to `edge_end`. The outward normal (for a CCW polygon)
  /// is the edge vector rotated 90° clockwise: `(dy, -dx)` where `(dx, dy) = edge_end - edge_start`.
  ///
  /// This is equivalent to `along_vector(p1, edge_normal, p2)` but avoids overflow
  /// that would occur from computing the normal explicitly.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  pub fn along_edge_normal<T>(
    p1: &[T; 2],
    edge_start: &[T; 2],
    edge_end: &[T; 2],
    p2: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_edge_normal_slope(edge_start, edge_end, p1, p2) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Locate `p2` in relation to the line perpendicular to an edge's outward normal.
  ///
  /// This is equivalent to `along_perp_vector(p1, edge_normal, p2)` but avoids overflow.
  pub fn along_perp_edge_normal<T>(
    p1: &[T; 2],
    edge_start: &[T; 2],
    edge_end: &[T; 2],
    p2: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_perp_edge_normal_slope(edge_start, edge_end, p1, p2) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Check if an edge's normal is on the positive side of the perpendicular to a reference edge's normal.
  fn along_perp_ref_edge_normal_edge<T>(
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    edge_start: &[T; 2],
    edge_end: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_edge_normals_dot(ref_edge_start, ref_edge_end, edge_start, edge_end) {
      Ordering::Greater => Orientation::ClockWise,
      Ordering::Less => Orientation::CounterClockWise,
      Ordering::Equal => Orientation::CoLinear,
    }
  }

  pub fn is_colinear(self) -> bool {
    matches!(self, Orientation::CoLinear)
  }

  pub fn is_ccw(self) -> bool {
    matches!(self, Orientation::CounterClockWise)
  }

  pub fn is_cw(self) -> bool {
    matches!(self, Orientation::ClockWise)
  }

  #[must_use]
  pub fn then(self, other: Orientation) -> Orientation {
    match self {
      Orientation::CoLinear => other,
      _ => self,
    }
  }

  pub fn break_ties(self, a: u32, b: u32, c: u32) -> SoS {
    match self {
      CounterClockWise => SoS::CounterClockWise,
      ClockWise => SoS::ClockWise,
      CoLinear => SoS::new(a, b, c),
    }
  }

  pub fn sos(self, other: SoS) -> SoS {
    match self {
      CounterClockWise => SoS::CounterClockWise,
      ClockWise => SoS::ClockWise,
      CoLinear => other,
    }
  }

  // pub fn around_origin<T>(q: &[T; 2], r: &[T; 2]) -> Orientation
  // where
  //   T: Ord + Mul<Output = T> + Clone + Extended,
  // {
  //   raw_arr_turn_origin(q, r)
  // }

  #[must_use]
  pub fn reverse(self) -> Orientation {
    match self {
      Orientation::CounterClockWise => Orientation::ClockWise,
      Orientation::ClockWise => Orientation::CounterClockWise,
      Orientation::CoLinear => Orientation::CoLinear,
    }
  }

  pub fn ccw_cmp_around_with<T>(
    vector: &Vector<T, 2>,
    p1: &[T; 2],
    p2: &[T; 2],
    p3: &[T; 2],
  ) -> Ordering
  where
    T: PolygonScalar,
  {
    let aq = Orientation::along_vector(p1, vector, p2);
    let ar = Orientation::along_vector(p1, vector, p3);
    // let on_zero = |d: &[T; 2]| {
    //   !((d[0] < p[0] && z[0].is_positive())
    //     || (d[1] < p[1] && z[1].is_positive())
    //     || (d[0] > p[0] && z[0].is_negative())
    //     || (d[1] > p[1] && z[1].is_negative()))
    // };
    let on_zero = |d: &[T; 2]| match Orientation::along_perp_vector(p1, vector, d) {
      CounterClockWise => false,
      ClockWise => true,
      CoLinear => true,
    };
    let cmp = || match Orientation::new(p1, p2, p3) {
      CounterClockWise => Ordering::Less,
      ClockWise => Ordering::Greater,
      CoLinear => Ordering::Equal,
    };
    match (aq, ar) {
      // Easy cases: Q and R are on either side of the line p->z:
      (CounterClockWise, ClockWise) => Ordering::Less,
      (ClockWise, CounterClockWise) => Ordering::Greater,
      // A CoLinear point may be in front of p->z (0 degree angle) or behind
      // it (180 degree angle). If the other point is clockwise, it must have an
      // angle greater than 180 degrees and must therefore be greater than the
      // colinear point.
      (CoLinear, ClockWise) => Ordering::Less,
      (ClockWise, CoLinear) => Ordering::Greater,

      // if Q and R are on the same side of P->Z then the most clockwise point
      // will have the smallest angle.
      (CounterClockWise, CounterClockWise) => cmp(),
      (ClockWise, ClockWise) => cmp(),

      // CoLinear points have an angle of either 0 degrees or 180 degrees. on_zero
      // can distinguish these two cases:
      //    on_zero(p) => 0 degrees.
      //   !on_zero(p) => 180 degrees.
      (CounterClockWise, CoLinear) => {
        if on_zero(p3) {
          Ordering::Greater // angle(r) = 0 & 0 < angle(q) < 180. Thus: Q > R
        } else {
          Ordering::Less // angle(r) = 180 & 0 < angle(q) < 180. Thus: Q < R
        }
      }
      (CoLinear, CounterClockWise) => {
        if on_zero(p2) {
          Ordering::Less
        } else {
          Ordering::Greater
        }
      }
      (CoLinear, CoLinear) => match (on_zero(p2), on_zero(p3)) {
        (true, true) => Ordering::Equal,
        (false, false) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
      },
    }
  }

  /// Compare angles around an origin point, where the reference direction is an edge's outward normal.
  ///
  /// This is equivalent to `ccw_cmp_around_with(edge_normal, p1, p2, p3)` but avoids overflow
  /// that would occur from computing the edge normal explicitly.
  ///
  /// The edge goes from `ref_edge_start` to `ref_edge_end`. The outward normal (for a CCW polygon)
  /// is the edge vector rotated 90° clockwise: `(dy, -dx)` where `(dx, dy) = edge_end - edge_start`.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  pub fn ccw_cmp_around_with_edge_normal<T>(
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    p1: &[T; 2],
    p2: &[T; 2],
    p3: &[T; 2],
  ) -> Ordering
  where
    T: PolygonScalar,
  {
    let aq = Orientation::along_edge_normal(p1, ref_edge_start, ref_edge_end, p2);
    let ar = Orientation::along_edge_normal(p1, ref_edge_start, ref_edge_end, p3);
    let on_zero =
      |d: &[T; 2]| match Orientation::along_perp_edge_normal(p1, ref_edge_start, ref_edge_end, d) {
        CounterClockWise => false,
        ClockWise => true,
        CoLinear => true,
      };
    let cmp = || match Orientation::new(p1, p2, p3) {
      CounterClockWise => Ordering::Less,
      ClockWise => Ordering::Greater,
      CoLinear => Ordering::Equal,
    };
    match (aq, ar) {
      // Easy cases: Q and R are on either side of the line p->z:
      (CounterClockWise, ClockWise) => Ordering::Less,
      (ClockWise, CounterClockWise) => Ordering::Greater,
      // A CoLinear point may be in front of p->z (0 degree angle) or behind
      // it (180 degree angle). If the other point is clockwise, it must have an
      // angle greater than 180 degrees and must therefore be greater than the
      // colinear point.
      (CoLinear, ClockWise) => Ordering::Less,
      (ClockWise, CoLinear) => Ordering::Greater,

      // if Q and R are on the same side of P->Z then the most clockwise point
      // will have the smallest angle.
      (CounterClockWise, CounterClockWise) => cmp(),
      (ClockWise, ClockWise) => cmp(),

      // CoLinear points have an angle of either 0 degrees or 180 degrees. on_zero
      // can distinguish these two cases:
      //    on_zero(p) => 0 degrees.
      //   !on_zero(p) => 180 degrees.
      (CounterClockWise, CoLinear) => {
        if on_zero(p3) {
          Ordering::Greater // angle(r) = 0 & 0 < angle(q) < 180. Thus: Q > R
        } else {
          Ordering::Less // angle(r) = 180 & 0 < angle(q) < 180. Thus: Q < R
        }
      }
      (CoLinear, CounterClockWise) => {
        if on_zero(p2) {
          Ordering::Less
        } else {
          Ordering::Greater
        }
      }
      (CoLinear, CoLinear) => match (on_zero(p2), on_zero(p3)) {
        (true, true) => Ordering::Equal,
        (false, false) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
      },
    }
  }

  /// Compare angles around an origin point where:
  /// - The reference direction is an edge's outward normal (ref_edge_start → ref_edge_end)
  /// - One comparison point is another edge's outward normal (edge_start → edge_end)
  /// - The other comparison point is an explicit direction vector
  ///
  /// This is a specialized version of `ccw_cmp_around_with` designed for the
  /// `extreme_in_direction` algorithm that avoids overflow when computing edge normals.
  ///
  /// Returns the ordering of the edge normal's angle compared to the direction's angle,
  /// both measured from the reference normal as the zero angle.
  ///
  /// For fixed-precision types (i8,i16,i32,i64,etc), this function is
  /// guaranteed to work for any input and never cause any arithmetic overflows.
  pub fn ccw_cmp_around_with_edge_normal_vs_direction<T>(
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    p1: &[T; 2],
    edge_start: &[T; 2],
    edge_end: &[T; 2],
    direction: &[T; 2],
  ) -> Ordering
  where
    T: PolygonScalar,
  {
    // p2 = edge normal (implicitly defined by edge_start, edge_end)
    // p3 = direction (explicit)

    // aq = orientation of edge_normal relative to reference_normal line through p1
    let aq = Orientation::along_ref_edge_normal_edge(
      ref_edge_start,
      ref_edge_end,
      edge_start,
      edge_end,
    );
    // ar = orientation of direction relative to reference_normal line through p1
    let ar = Orientation::along_edge_normal_point(p1, ref_edge_start, ref_edge_end, direction);

    // on_zero_edge = is edge_normal at angle 0 (in front) or 180 (behind)?
    let on_zero_edge = || match Orientation::along_perp_ref_edge_normal_edge(
      ref_edge_start,
      ref_edge_end,
      edge_start,
      edge_end,
    ) {
      CounterClockWise => false,
      ClockWise => true,
      CoLinear => true,
    };

    // on_zero_dir = is direction at angle 0 (in front) or 180 (behind)?
    let on_zero_dir = || match Orientation::along_perp_edge_normal_point(
      p1,
      ref_edge_start,
      ref_edge_end,
      direction,
    ) {
      CounterClockWise => false,
      ClockWise => true,
      CoLinear => true,
    };

    // cmp = direct orientation comparison of edge_normal vs direction around p1
    let cmp = || {
      // We need Orientation::new(p1, edge_normal, direction)
      // This returns CCW if direction is CCW (left) of p1→edge_normal,
      // which means edge_normal has a smaller angle than direction.
      //
      // cmp_edge_normal_vs_direction returns sign(edge_normal × direction):
      // - Greater (positive): direction is CCW from edge_normal → edge_normal < direction → Less
      // - Less (negative): direction is CW from edge_normal → edge_normal > direction → Greater
      // - Equal: collinear
      //
      // This mapping matches the original ccw_cmp_around_with logic where:
      // - CCW → Less (q comes before r)
      // - CW → Greater (q comes after r)
      match Orientation::cmp_edge_normal_vs_direction(edge_start, edge_end, direction) {
        Ordering::Greater => Ordering::Less, // edge_normal × direction > 0 → edge_normal < direction
        Ordering::Less => Ordering::Greater, // edge_normal × direction < 0 → edge_normal > direction
        Ordering::Equal => Ordering::Equal,
      }
    };

    match (aq, ar) {
      // Easy cases: Q and R are on either side of the line p->z:
      (CounterClockWise, ClockWise) => Ordering::Less,
      (ClockWise, CounterClockWise) => Ordering::Greater,
      // A CoLinear point may be in front of p->z (0 degree angle) or behind
      // it (180 degree angle). If the other point is clockwise, it must have an
      // angle greater than 180 degrees and must therefore be greater than the
      // colinear point.
      (CoLinear, ClockWise) => Ordering::Less,
      (ClockWise, CoLinear) => Ordering::Greater,

      // if Q and R are on the same side of P->Z then the most clockwise point
      // will have the smallest angle.
      (CounterClockWise, CounterClockWise) => cmp(),
      (ClockWise, ClockWise) => cmp(),

      // CoLinear points have an angle of either 0 degrees or 180 degrees.
      (CounterClockWise, CoLinear) => {
        if on_zero_dir() {
          Ordering::Greater
        } else {
          Ordering::Less
        }
      }
      (CoLinear, CounterClockWise) => {
        if on_zero_edge() {
          Ordering::Less
        } else {
          Ordering::Greater
        }
      }
      (CoLinear, CoLinear) => match (on_zero_edge(), on_zero_dir()) {
        (true, true) => Ordering::Equal,
        (false, false) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
      },
    }
  }

  /// Orientation of a point relative to a line defined by an edge's outward normal.
  /// This is `along_edge_normal` but for an explicit point instead of another edge.
  fn along_edge_normal_point<T>(
    p1: &[T; 2],
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    point: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    // The edge normal is (ref_edge_end[1] - ref_edge_start[1], ref_edge_start[0] - ref_edge_end[0])
    // along_vector(p1, normal, point) computes orientation of point relative to line p1→(p1+normal)
    // This is: sign((p1 - point) × normal) = sign((p1.x - point.x) * normal.y - (p1.y - point.y) * normal.x)
    // = sign((p1.x - point.x) * (ref_edge_start[0] - ref_edge_end[0]) - (p1.y - point.y) * (ref_edge_end[1] - ref_edge_start[1]))
    match T::cmp_edge_normal_slope(ref_edge_start, ref_edge_end, p1, point) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Orientation of a point relative to a line perpendicular to an edge's outward normal.
  fn along_perp_edge_normal_point<T>(
    p1: &[T; 2],
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    point: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_perp_edge_normal_slope(ref_edge_start, ref_edge_end, p1, point) {
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
      Ordering::Greater => Orientation::CounterClockWise,
    }
  }

  /// Orientation of an edge's outward normal relative to another edge's normal line.
  fn along_ref_edge_normal_edge<T>(
    ref_edge_start: &[T; 2],
    ref_edge_end: &[T; 2],
    edge_start: &[T; 2],
    edge_end: &[T; 2],
  ) -> Orientation
  where
    T: PolygonScalar,
  {
    match T::cmp_edge_normals_cross(ref_edge_start, ref_edge_end, edge_start, edge_end) {
      Ordering::Greater => Orientation::CounterClockWise,
      Ordering::Less => Orientation::ClockWise,
      Ordering::Equal => Orientation::CoLinear,
    }
  }

  /// Compare an edge normal against a direction vector.
  fn cmp_edge_normal_vs_direction<T>(
    edge_start: &[T; 2],
    edge_end: &[T; 2],
    direction: &[T; 2],
  ) -> Ordering
  where
    T: PolygonScalar,
  {
    T::cmp_edge_normal_cross_direction(edge_start, edge_end, direction)
  }
}

// https://arxiv.org/abs/math/9410209
// Simulation of Simplicity.
// Break ties (ie colinear orientations) in an arbitrary but consistent way.
impl SoS {
  // p: Point::new([a, 2^a])
  // q: Point::new([b, 2^b])
  // r: Point::new([c, 2^c])
  // new(a,b,c) == Orientation::new(p, q, r)
  pub fn new(a: u32, b: u32, c: u32) -> SoS {
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(c, a);

    // Combinations:
    //               a<b a<c c<b
    // b a c => CW    _   X   _
    // c b a => CW    _   _   X
    // a c b => CW    X   X   X
    // b c a => CCW   _   _   _
    // c a b => CCW   X   _   X
    // a b c => CCW   X   X   _

    let ab = a < b;
    let ac = a < c;
    let cb = c < b;
    if ab ^ ac ^ cb {
      SoS::ClockWise
    } else {
      SoS::CounterClockWise
    }
    // if a < b {
    //   if a < c && c < b {
    //     SoS::ClockWise // a c b
    //   } else {
    //     SoS::CounterClockWise // a b c, c a b
    //   }
    // } else if b < c && c < a {
    //   SoS::CounterClockWise // b c a
    // } else {
    //   SoS::ClockWise // b a c, c b a
    // }
  }

  pub fn orient(self) -> Orientation {
    match self {
      SoS::CounterClockWise => Orientation::CounterClockWise,
      SoS::ClockWise => Orientation::ClockWise,
    }
  }

  #[must_use]
  pub fn reverse(self) -> SoS {
    match self {
      SoS::CounterClockWise => SoS::ClockWise,
      SoS::ClockWise => SoS::CounterClockWise,
    }
  }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
  use super::*;

  use crate::data::Point;
  use num::BigInt;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[test]
  fn orientation_limit_1() {
    PolygonScalar::cmp_slope(
      &[i8::MAX, i8::MAX],
      &[i8::MIN, i8::MIN],
      &[i8::MIN, i8::MIN],
    );
  }

  #[test]
  fn cmp_slope_1() {
    assert_eq!(
      PolygonScalar::cmp_slope(&[0i8, 0], &[1, 1], &[2, 2],),
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
    let options = &[i8::MIN, i8::MAX, 0, -10, 10];
    for [a, b, c, d, e, f] in crate::utils::permutations([options; 6]) {
      PolygonScalar::cmp_slope(&[a, b], &[c, d], &[e, f]);
    }
  }

  #[test]
  fn cmp_around_1() {
    use num_bigint::*;
    let pt1 = [BigInt::from(0), BigInt::from(0)];
    let pt2 = [BigInt::from(-1), BigInt::from(1)];
    // let pt2 = [BigInt::from(-717193444810564826_i64), BigInt::from(1)];
    let vector = Vector([BigInt::from(1), BigInt::from(0)]);

    assert_eq!(
      Orientation::ccw_cmp_around_with(&vector, &pt1, &pt2, &pt1),
      Ordering::Greater
    );
  }

  #[test]
  fn sos_unit1() {
    assert_eq!(SoS::new(0, 1, 2), SoS::CounterClockWise)
  }

  #[test]
  #[should_panic]
  fn sos_unit2() {
    SoS::new(0, 0, 1);
  }

  #[test]
  fn sos_unit3() {
    assert_eq!(SoS::new(99, 0, 1), SoS::CounterClockWise);
  }

  #[proptest]
  fn sos_eq_prop(a: u8, b: u8, c: u8) {
    if a != b && b != c && c != a {
      let (a, b, c) = (a as u32, b as u32, c as u32);
      let one = &BigInt::from(1);
      let big_a = BigInt::from(a);
      let big_b = BigInt::from(b);
      let big_c = BigInt::from(c);
      let p = Point::new([big_a, one << a]);
      let q = Point::new([big_b, one << b]);
      let r = Point::new([big_c, one << c]);
      prop_assert_eq!(SoS::new(a, b, c).orient(), Orientation::new(&p, &q, &r));
    }
  }

  #[proptest]
  fn sos_rev_prop(a: u32, b: u32, c: u32) {
    if a != b && b != c && c != a {
      prop_assert_eq!(SoS::new(a, b, c), SoS::new(c, b, a).reverse());
      prop_assert_eq!(SoS::new(a, b, c), SoS::new(a, c, b).reverse());
      prop_assert_eq!(SoS::new(a, b, c), SoS::new(b, a, c).reverse());
      prop_assert_eq!(SoS::new(a, b, c), SoS::new(b, c, a));
    }
  }
}
