// https://en.wikipedia.org/wiki/Monotone_polygon
use crate::data::{Cursor, Point, Polygon, Vector};
use crate::{Error, Orientation, PolygonScalar, TotalOrd};

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Bound::*;

/// Check if the given polygon is monotone with respect to given direction
pub fn is_monotone<T>(poly: &Polygon<T>, direction: &Vector<T, 2>) -> bool
where
  T: PolygonScalar,
{
  // We can only check polygons without. It would be nice to enforce this with types.
  assert_eq!(poly.rings.len(), 1);

  let cmp_cursors =
    |a: &Cursor<'_, T>, b: &Cursor<'_, T>| direction.cmp_along(a, b).then_with(|| a.total_cmp(b));
  // XXX: Is there a way to get both the min and max element at the same time?
  let max_cursor = {
    match poly.iter_boundary().max_by(cmp_cursors) {
      Some(c) => c,
      None => return false,
    }
  };
  let min_cursor = {
    match poly.iter_boundary().min_by(cmp_cursors) {
      Some(c) => c,
      None => return false,
    }
  };

  // All points going counter-clockwise from min_cursor to max_cursor must be
  // less-than or equal to the next point in the chain along the direction vector.
  for pt in min_cursor.to(Excluded(max_cursor)) {
    if direction.cmp_along(&pt, &pt.next()) == Ordering::Greater {
      return false;
    }
  }

  // Walking down the other chain, the condition is opposite: All points
  // must be greater-than or equal to the next point in the chain along the direction vector.
  for pt in max_cursor.to(Excluded(min_cursor)) {
    if direction.cmp_along(&pt, &pt.next()) == Ordering::Less {
      return false;
    }
  }

  true
}

/// Generates a monotone polygon from given points with respect to given direction
pub fn new_monotone_polygon<T>(
  mut points: Vec<Point<T, 2>>,
  direction: &Vector<T, 2>,
) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar,
{
  // First compare along the direction vector.
  // If two points are the same distance along the vector, compare their X and Y components.
  points.sort_by(|prev, curr| {
    direction
      .cmp_along(prev, curr)
      .then_with(|| prev.total_cmp(curr))
  });

  points.dedup();
  if points.len() < 3 {
    return Err(Error::InsufficientVertices);
  }

  let (min_point, max_point) = (
    points.first().unwrap().clone(),
    points.last().unwrap().clone(),
  );

  // The polygon is made up of two chains running from 'min_point' to
  // 'max_point': One on the clockwise side of the line between them and one on
  // the counter-clockwise side. Points that lie exactly on that line may go on
  // either chain with one caveat: A chain without any points of its own is just
  // the straight line from 'min_point' to 'max_point', and a co-linear point on
  // the other chain would then sit on top of that line, making the polygon
  // self-intersecting. So, co-linear points are always placed on the chain that
  // has no points of its own.
  let last = points.len() - 1;
  let colinear_side = if points[1..last]
    .iter()
    .any(|pt| Orientation::new(&min_point, &max_point, pt) == Orientation::CounterClockWise)
  {
    Orientation::ClockWise
  } else {
    Orientation::CounterClockWise
  };

  let mut polygon_points: VecDeque<Point<T, 2>> = VecDeque::new();

  while let Some(curr) = points.pop() {
    let side = match Orientation::new(&min_point, &max_point, &curr) {
      Orientation::CoLinear => colinear_side,
      side => side,
    };
    match side {
      Orientation::ClockWise => polygon_points.push_front(curr),
      _ => polygon_points.push_back(curr),
    }
  }
  let vec = Vec::from(polygon_points);

  Polygon::new(vec)
}

//testing
#[cfg(test)]
mod monotone_testing {
  use super::*;
  use crate::Orientation;
  use crate::data::{Point, Polygon, PolygonConvex, Vector};
  use proptest::prelude::*;
  use std::collections::BTreeSet;
  use test_strategy::proptest;

  #[proptest]
  fn convex_polygon_is_monotone(convex_polygon: PolygonConvex<i8>, direction: Vector<i8, 2>) {
    prop_assert!(is_monotone(convex_polygon.polygon(), &direction));
  }

  #[test]
  //ToDo: Find a way to proptest the Non-monotone case
  fn non_y_monotone() {
    let polygon = Polygon::new(vec![
      Point::new([0, 1]),
      Point::new([1, 2]),
      Point::new([1, -2]),
      Point::new([0, -1]),
      Point::new([-1, -2]),
      Point::new([-1, 2]),
    ])
    .unwrap();
    assert!(!is_monotone(&polygon, &Vector::from(Point::new([0, 1]))));
  }

  #[test]
  fn monotone_mountain() {
    let polygon = Polygon::new(vec![
      Point::new([0, 3]),
      Point::new([1, 2]),
      Point::new([1, -2]),
      Point::new([0, -3]),
    ])
    .unwrap();
    assert!(is_monotone(&polygon, &Vector::from(Point::new([0, 1]))));
  }

  // Regression test: Points that lie on the line between the extreme points
  // used to be placed on the same chain as (0,0), leaving the other chain as a
  // bare line from (113,-8) to (23,127) with (111,-5) sitting on top of it.
  #[test]
  fn colinear_points_on_empty_chain() {
    let points = vec![
      Point::new([111, -5]),
      Point::new([23, 127]),
      Point::new([0, 0]),
      Point::new([113, -8]),
    ];
    let direction = Vector([0, 1]);
    let polygon = new_monotone_polygon(points, &direction).unwrap();
    assert!(is_monotone(&polygon, &direction));
    assert_eq!(polygon.validate().err(), None);
  }

  #[proptest]
  fn monotone_is_monotone_prop(points: Vec<Point<i8, 2>>, direction: Vector<i8, 2>) {
    if let Ok(p) = new_monotone_polygon(points, &direction) {
      prop_assert!(is_monotone(&p, &direction));
      prop_assert_eq!(p.validate().err(), None);
    }
  }

  #[proptest]
  fn valid_monotone(points: Vec<Point<i8, 2>>, direction: Vector<i8, 2>) {
    // dedup points
    let mut points = points;
    let mut set = BTreeSet::new();
    points.retain(|pt| set.insert(*pt));
    // If we have at least three, non-colinear points, then we must be able to
    // create a monotone polygon.
    if !points
      .windows(3)
      .all(|window| Orientation::new(&window[0], &window[1], &window[2]).is_colinear())
    {
      let p = new_monotone_polygon(points, &direction).unwrap();
      prop_assert!(is_monotone(&p, &direction));
      prop_assert_eq!(p.validate().err(), None);
    }
  }
}
