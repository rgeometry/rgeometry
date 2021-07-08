// https://en.wikipedia.org/wiki/Monotone_polygon
use crate::data::{Cursor, Point, Polygon, Vector};
use crate::{Error, Orientation, PolygonScalar};
use rand::SeedableRng;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::process::exit;
use std::vec;

///Check if the given polyon is monotone with resprect to given direction
pub fn is_monotone<T>(poly: &Polygon<T>, direction: &Vector<T, 2>) -> bool
where
  T: PolygonScalar,
{
  // We can only check polygons without. It would be nice to enforce this with types.
  assert_eq!(poly.rings.len(), 1);

  let cmp_cursors = |a: &Cursor<'_, T>, b: &Cursor<'_, T>| direction.cmp_along(a, b).then(a.cmp(b));
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
  for pt in min_cursor.to(max_cursor) {
    if direction.cmp_along(&pt, &pt.next()) == Ordering::Greater {
      return false;
    }
  }

  // Walking down the other chain, the condition is opposite: All points
  // must be greater-than or equal to the next point in the chain along the direction vector.
  for pt in max_cursor.to(min_cursor) {
    if direction.cmp_along(&pt, &pt.next()) == Ordering::Less {
      return false;
    }
  }

  true

  // let mut points = poly.points.clone();
  // let max_index = points
  //   .iter()
  //   .enumerate()
  //   .max_by(|(_, curr_val), (_, nxt_val)| {
  //     direction
  //       .cmp_along(curr_val, nxt_val)
  //       .then(curr_val.cmp(nxt_val))
  //   })
  //   .map(|(idx, _)| idx)
  //   .unwrap();

  // points.rotate_left(max_index);

  // let min_index = points
  //   .iter()
  //   .enumerate()
  //   .min_by(|(_, curr_val), (_, nxt_val)| {
  //     direction
  //       .cmp_along(curr_val, nxt_val)
  //       .then(curr_val.cmp(nxt_val))
  //   })
  //   .map(|(idx, _)| idx)
  //   .unwrap();

  // Ok(
  //   points[..min_index]
  //     .windows(2)
  //     .all(|pair| direction.cmp_along(&pair[1], &pair[0]) != Ordering::Greater)
  //     && points[min_index..]
  //       .windows(2)
  //       .all(|pair| direction.cmp_along(&pair[1], &pair[0]) != Ordering::Less),
  // )
}

/// Generates a monotone polygon from given points with respect to given direction
pub fn form_monotone_polygon<T>(
  mut points: Vec<Point<T, 2>>,
  direction: &Vector<T, 2>,
) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar,
{
  if points.len() < 3 {
    return Err(Error::InsufficientVertices);
  }

  // First compare along the direction vector.
  // Then compare the x-component.
  // Then compare the y-component.
  points.sort_by(|prev, curr| direction.cmp_along(prev, curr).then(prev.cmp(&curr)));
  let (min_point, max_point) = (
    points.first().unwrap().clone(),
    points.last().unwrap().clone(),
  );

  let mut polygon_points: VecDeque<Point<T, 2>> = VecDeque::new();

  while let Some(curr) = points.pop() {
    match Orientation::new(&min_point, &max_point, &curr) {
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
  use crate::algorithms::monotone_polygon;
  use crate::data::{Point, Polygon, PolygonConvex, Vector};
  use crate::Orientation;
  use proptest::collection::vec;
  use proptest::prelude::*;
  use rand::prelude::SliceRandom;
  use rand::SeedableRng;
  use std::assert;
  use test_strategy::proptest;

  #[proptest]
  fn convex_polygon_is_montone(convex_polygon: PolygonConvex<i8>, direction: Vector<i8, 2>) {
    prop_assert!(monotone_polygon::is_monotone(
      &convex_polygon.polygon(),
      &direction
    ));
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
    assert!(!monotone_polygon::is_monotone(
      &polygon,
      &Vector::from(Point::new([0, 1]))
    ));
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
    assert!(monotone_polygon::is_monotone(
      &polygon,
      &Vector::from(Point::new([0, 1]))
    ));
  }

  #[proptest]
  fn generate_monotone(points: Vec<Point<i8, 2>>, direction: Vector<i8, 2>) {
    if let Ok(p) = monotone_polygon::form_monotone_polygon(points, &direction) {
      prop_assert!(monotone_polygon::is_monotone(&p, &direction));
    }
  }
}
