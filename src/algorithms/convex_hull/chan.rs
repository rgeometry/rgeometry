use std::cmp::Ordering;

use super::graham_scan;
use crate::data::{Point, Polygon, PolygonConvex};
use crate::{Error, Orientation, PolygonScalar, TotalOrd};

// https://en.wikipedia.org/wiki/Chan%27s_algorithm

// Properties:
//    No panics.
//    All Ok results are valid convex polygons.
//    No points are outside the resulting convex polygon.
/// Convex hull of a set of points.
///
/// [Chan's][wiki] algorithm for finding the smallest convex polygon which
/// contains all the given points.
///
/// # Errors
/// Will return an error iff the input set contains less than three distinct points.
///
/// # Properties
/// * No points from the input set will be outside the returned convex polygon.
/// * All vertices in the convex polygon are from the input set.
///
/// # Time complexity
/// $O(n \log h)$ where h is the number of points on the convex hull.
///
/// # Examples
///
/// ```rust
/// # pub fn main() {
/// # use rgeometry::algorithms::convex_hull::chan;
/// # use rgeometry::data::Point;
/// # use rgeometry::Error;
/// let empty_set: Vec<Point<i32,2>> = vec![];
/// assert_eq!(
///   chan::convex_hull(empty_set).err(),
///   Some(Error::InsufficientVertices))
/// # }
/// ```
///
/// ```rust
/// # pub fn main() {
/// # use rgeometry::algorithms::convex_hull::chan;
/// # use rgeometry::data::Point;
/// # use rgeometry::Error;
/// let dups = vec![Point::new([0,0])].repeat(3);
/// assert_eq!(
///   chan::convex_hull(dups).err(),
///   Some(Error::InsufficientVertices))
/// # }
/// ```
///
/// [wiki]: https://en.wikipedia.org/wiki/Chan%27s_algorithm
pub fn convex_hull<T>(pts: Vec<Point<T>>) -> Result<PolygonConvex<T>, Error>
where
  T: PolygonScalar,
{
  let n = pts.len();
  if n < 3 {
    return Err(Error::InsufficientVertices);
  }

  let start = start_point(&pts)?;

  let mut t: u32 = 1;
  loop {
    let m = block_size(t, n);
    if let Some(hull) = attempt(&pts, &start, m) {
      if hull.len() < 3 {
        return Err(Error::InsufficientVertices);
      }
      return Ok(PolygonConvex::new_unchecked(Polygon::new_unchecked(hull)));
    }
    t += 1;
  }
}

// m = min(n, 2^(2^t)), without ever attempting a shift that would overflow.
fn block_size(t: u32, n: usize) -> usize {
  let exponent: u64 = 1u64 << t.min(6);
  if exponent >= usize::BITS as u64 {
    n
  } else {
    (1usize << exponent).min(n)
  }
}

// Builds a local hull per group of `m` points, then gift-wraps across
// groups. Returns None if the hull doesn't close within `m` points.
fn attempt<T>(pts: &[Point<T>], start: &Point<T>, m: usize) -> Option<Vec<Point<T>>>
where
  T: PolygonScalar,
{
  let groups: Vec<Vec<Point<T>>> = pts
    .chunks(m)
    .map(|chunk| local_hull(chunk.to_vec()))
    .collect();

  let (mut cur_group, mut cur_local) = groups
    .iter()
    .enumerate()
    .find_map(|(gi, grp)| grp.iter().position(|pt| pt == start).map(|li| (gi, li)))?;

  let first_point = groups[cur_group][cur_local].clone();
  let mut hull = vec![first_point.clone()];
  let mut cur_point = first_point.clone();

  loop {
    let mut best: Option<(usize, usize)> = None;
    for (gi, grp) in groups.iter().enumerate() {
      let li = if gi == cur_group {
        (cur_local + 1) % grp.len()
      } else {
        tangent_index(grp, &cur_point)
      };
      match best {
        None => best = Some((gi, li)),
        Some((best_gi, best_li)) => {
          if better_candidate(&cur_point, &groups[best_gi][best_li], &grp[li]) {
            best = Some((gi, li));
          }
        }
      }
    }
    let (best_group, best_local) = best?;
    cur_group = best_group;
    cur_local = best_local;
    let next_point = &groups[cur_group][cur_local];

    if *next_point == first_point {
      return Some(hull);
    }
    hull.push(next_point.clone());
    cur_point = next_point.clone();
    if hull.len() > m {
      return None;
    }
  }
}

// Convex hull of a group, as points in counter-clockwise order.
fn local_hull<T>(group: Vec<Point<T>>) -> Vec<Point<T>>
where
  T: PolygonScalar,
{
  if group.len() < 3 {
    return group;
  }
  let backup = group.clone();
  match graham_scan::convex_hull(group) {
    Ok(hull) => hull.iter().cloned().collect(),
    Err(_) => collinear_extremes(backup),
  }
}

// The two extreme points of a colinear (or coincident) point set.
fn collinear_extremes<T>(group: Vec<Point<T>>) -> Vec<Point<T>>
where
  T: PolygonScalar,
{
  let first = group[0].clone();
  match group.iter().find(|pt| **pt != first) {
    None => vec![first],
    Some(other) => {
      let direction = other - &first;
      let min = group
        .iter()
        .min_by(|a, b| direction.cmp_along(a, b))
        .unwrap();
      let max = group
        .iter()
        .max_by(|a, b| direction.cmp_along(a, b))
        .unwrap();
      vec![min.clone(), max.clone()]
    }
  }
}

// Index of the vertex q in `hull` such that every other vertex lies to the
// left of ray p -> q. Requires p to not be a point of `hull` itself.
//
// hull spans less than 180 degrees from external point p, so better_candidate
// gives a consistent ordering over all of it with a single peak (the answer)
// and a single valley. Binary search for the far end of "better than hull[0]"
// brackets the peak, then a bitonic search pins it down exactly.
fn tangent_index<T>(hull: &[Point<T>], p: &Point<T>) -> usize
where
  T: PolygonScalar,
{
  let n = hull.len();
  if n <= 2 {
    let mut best = 0;
    for i in 1..n {
      if better_candidate(p, &hull[best], &hull[i]) {
        best = i;
      }
    }
    return best;
  }

  let dir: i64 = if better_candidate(p, &hull[0], &hull[1]) {
    1
  } else if better_candidate(p, &hull[0], &hull[n - 1]) {
    -1
  } else {
    // hull[0] is already the tangent point.
    return 0;
  };
  let idx = |k: i64| -> usize { (dir * k).rem_euclid(n as i64) as usize };

  // Far end of the arc of vertices better than hull[0].
  let mut lo: i64 = 1;
  let mut hi: i64 = n as i64 - 1;
  while lo < hi {
    let mid = lo + (hi - lo + 1) / 2;
    if better_candidate(p, &hull[0], &hull[idx(mid)]) {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }
  let bracket_end = lo;

  // Bitonic search for the exact peak within that bracket.
  let mut lo = 1i64;
  let mut hi = bracket_end;
  while lo < hi {
    let mid = lo + (hi - lo) / 2;
    if better_candidate(p, &hull[idx(mid)], &hull[idx(mid + 1)]) {
      lo = mid + 1;
    } else {
      hi = mid;
    }
  }
  idx(lo)
}

// Is `candidate` a better next-hull point than `current`, given pivot `p`?
fn better_candidate<T>(p: &Point<T>, current: &Point<T>, candidate: &Point<T>) -> bool
where
  T: PolygonScalar,
{
  match Point::orient(p, current, candidate) {
    Orientation::ClockWise => true,
    Orientation::CoLinear => p.cmp_distance_to(current, candidate) == Ordering::Less,
    Orientation::CounterClockWise => false,
  }
}

// Finds the bottommost, then leftmost, point.
// O(n)
fn start_point<T>(pts: &[Point<T>]) -> Result<Point<T>, Error>
where
  T: PolygonScalar,
{
  pts
    .iter()
    .min_by(|a, b| TotalOrd::total_cmp(&(a.y_coord(), a.x_coord()), &(b.y_coord(), b.x_coord())))
    .cloned()
    .ok_or(Error::InsufficientVertices)
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
  use super::*;
  use crate::data::PointLocation;
  use crate::testing::*;

  use claims::assert_ok;
  use num_bigint::BigInt;

  use proptest::collection::*;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[test]
  fn convex_hull_colinear() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([3, 0]),
      Point::new([4, 0]),
      Point::new([1, 1]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_colinear_rev() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([0, 9]),
      Point::new([0, 8]),
      Point::new([0, 7]),
      Point::new([0, 6]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_dups() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 2]),
      Point::new([2, 2]),
      Point::new([5, 1]),
      Point::new([5, 1]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_insufficient_dups() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([0, 0]),
      Point::new([2, 2]),
      Point::new([2, 2]),
      Point::new([0, 0]),
      Point::new([2, 2]),
    ];
    assert_eq!(convex_hull(points).err(), Some(Error::InsufficientVertices));
  }

  #[test]
  fn convex_hull_invalid() {
    let points: Vec<Point<i64>> = vec![
      Point { array: [0, 0] },
      Point { array: [100, 0] },
      Point { array: [50, 1] },
      Point { array: [40, 1] },
      Point { array: [0, 100] },
    ];
    let points: Vec<Point<BigInt, 2>> = points.into_iter().map(|pt| pt.cast()).collect();
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn unit_1() {
    let points: Vec<Point<BigInt>> = vec![
      Point::new([0, 0]).into(),
      Point::new([-1, 1]).into(),
      Point::new([0, 1]).into(),
      Point::new([-717193444810564826, 1]).into(),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn unit_2() {
    let points: Vec<Point<i8>> = vec![
      Point::new([0, 0]),
      Point::new([0, -10]),
      Point::new([-13, 0]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  // A point set larger than any single group at t=1 (m=4), to exercise the
  // merge/retry logic across multiple phases.
  #[test]
  fn convex_hull_many_points() {
    let mut points = vec![
      Point::new([0, 0]),
      Point::new([100, 0]),
      Point::new([100, 100]),
      Point::new([0, 100]),
    ];
    for i in 1..99 {
      points.push(Point::new([i, i % 3]));
    }
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[proptest]
  fn convex_hull_prop(#[strategy(vec(any_r(), 0..100))] pts: Vec<Point<BigInt>>) {
    if let Ok(poly) = convex_hull(pts.clone()) {
      // Prop #1: Results are valid.
      prop_assert_eq!(poly.validate().err(), None);
      // Prop #2: No points from the input set are outside the polygon.
      for pt in pts.iter() {
        prop_assert_ne!(poly.locate(pt), PointLocation::Outside)
      }
      // Prop #3: All vertices are in the input set.
      for pt in poly.iter() {
        prop_assert!(pts.contains(pt))
      }
    }
  }

  #[proptest]
  fn convex_hull_prop_i8(#[strategy(vec(any::<Point<i8>>(), 0..100))] pts: Vec<Point<i8>>) {
    if let Ok(poly) = convex_hull(pts.clone()) {
      // Prop #1: Results are valid.
      prop_assert_eq!(poly.validate().err(), None);
      // Prop #2: No points from the input set are outside the polygon.
      for pt in pts.iter() {
        prop_assert_ne!(poly.locate(pt), PointLocation::Outside)
      }
      // Prop #3: All vertices are in the input set.
      for pt in poly.iter() {
        prop_assert!(pts.contains(pt))
      }
    }
  }

  // Cross-check against graham_scan: both algorithms must agree on which
  // points end up on the hull, regardless of how each one got there.
  #[proptest]
  fn matches_graham_scan_i8(#[strategy(vec(any::<Point<i8>>(), 0..100))] pts: Vec<Point<i8>>) {
    let chan_result = convex_hull(pts.clone());
    let graham_result = graham_scan::convex_hull(pts);
    match (chan_result, graham_result) {
      (Ok(chan_poly), Ok(graham_poly)) => {
        let mut chan_pts: Vec<_> = chan_poly.iter().cloned().collect();
        let mut graham_pts: Vec<_> = graham_poly.iter().cloned().collect();
        chan_pts.sort();
        graham_pts.sort();
        prop_assert_eq!(chan_pts, graham_pts);
      }
      (Err(chan_err), Err(graham_err)) => prop_assert_eq!(chan_err, graham_err),
      (chan_result, graham_result) => {
        prop_assert!(false, "mismatch: {:?} vs {:?}", chan_result, graham_result)
      }
    }
  }
}
