// https://en.wikipedia.org/wiki/Monotone_polygon
use crate::data::{Point, Polygon, Vector};
use crate::{Error, Orientation, PolygonScalar};
use rand::SeedableRng;

use std::collections::VecDeque;
use std::vec;

// fn is_monotone(poly: Polygon, direction: Vector) -> bool {
//   todo!()
// }

/// Checks if points form a y-monotone polygon, returns its paths if true
pub fn get_y_monotone_polygons<T>(points: &[Point<T, 2>]) -> Result<[Vec<&Point<T, 2>>; 2], Error>
where
  T: PolygonScalar,
{
  if points.len() < 3 {
    return Err(Error::InsufficientVertices);
  }

  let mut monotone_polygons: [Vec<&Point<T, 2>>; 2] = Default::default();
  let mut reordered_polygon: Vec<&Point<T, 2>> = Vec::new();

  let max_y_index = points
    .iter()
    .enumerate()
    .max_by(|(_, curr), (_, nxt)| curr.array[1].cmp(&nxt.array[1]))
    .map(|(idx, _)| idx)
    .expect("Empty Polygon");

  //Reorder polygon on Y index
  for point in &points[max_y_index..] {
    reordered_polygon.push(point);
  }
  for point in &points[..max_y_index] {
    reordered_polygon.push(point);
  }

  //Get monotone polygons if any
  let mut idx = 1;
  while idx < reordered_polygon.len() {
    let curr = reordered_polygon[idx];
    let prev = reordered_polygon[idx - 1];

    if curr.array[1] > prev.array[1] {
      break;
    }
    idx += 1;
  }
  monotone_polygons[0].extend_from_slice(&reordered_polygon[0..idx]);

  for i in idx..reordered_polygon.len() {
    let curr = reordered_polygon[i];
    let prev = reordered_polygon[i - 1];
    if curr.array[1] < prev.array[1] {
      return Err(Error::InsufficientVertices); // ToDo: replace with proper error
    }
  }
  monotone_polygons[1].extend_from_slice(&reordered_polygon[idx - 1..]);
  monotone_polygons[1].push(&points.last().unwrap());
  Ok(monotone_polygons)
}

/// Generates a monotone polygon from given points around given direction
pub fn form_monotone_polygon<T>(
  direction: Vector<T, 2>,
  mut points: Vec<Point<T, 2>>,
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

  // points.reverse();
  while let Some(curr) = points.pop() {
    // let curr = points.remove(0);
    // let curr = points.pop();
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
  fn convex_polygon_is_montone(convex_polygon: PolygonConvex<i8>) {
    prop_assert_eq!(
      monotone_polygon::get_y_monotone_polygons(&convex_polygon.points).err(),
      None
    );
  }

  #[test]
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
    let res = monotone_polygon::get_y_monotone_polygons(&polygon.points);
    assert!(res.is_err());
  }
  #[test]
  fn convex_polygon_monotone_paths() {
    let polygon = Polygon::new(vec![
      Point::new([0, 3]),
      Point::new([1, 2]),
      Point::new([1, -2]),
      Point::new([0, -3]),
      Point::new([-1, -2]),
      Point::new([-1, 2]),
    ])
    .unwrap();
    let res = monotone_polygon::get_y_monotone_polygons(&polygon.points);
    assert!(res.is_ok());
    let res_paths = res.unwrap();
    assert_eq!(res_paths[0].len(), 4);
    assert_eq!(res_paths[1].len(), 4);
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
    let res = monotone_polygon::get_y_monotone_polygons(&polygon.points);
    assert!(res.is_ok());
    let res_paths = res.unwrap();
    assert_eq!(res_paths[0].len(), 4);
    assert_eq!(res_paths[1].len(), 2);
  }

  #[proptest]
  fn generate_y_monotone(polygon: Polygon<i8>) {
    let mut points = polygon.points;
    points.shuffle(&mut rand::thread_rng());

    let res = monotone_polygon::form_monotone_polygon(Vector::from(Point::new([0, 1])), points);

    prop_assert!(res.is_ok());
  }
  #[test]
  fn generate_y_monotone_1() {
    let polygon = Polygon::new(vec![
      Point { array: [-54, 94] },
      Point { array: [-15, 55] },
      Point { array: [75, -1] },
      Point { array: [94, -4] },
      Point { array: [98, -9] },
      Point { array: [16, -5] },
      Point { array: [33, -33] },
      Point { array: [-30, -7] },
      Point { array: [-57, 30] },
      Point { array: [-60, 3] },
      Point { array: [-80, -55] },
      Point { array: [-23, -66] },
      Point { array: [-91, -59] },
      Point { array: [-105, -90] },
      Point { array: [-17, -67] },
      Point { array: [-29, -37] },
      Point { array: [2, -66] },
      Point { array: [25, -79] },
      Point { array: [24, -73] },
      Point { array: [25, -70] },
      Point { array: [52, -109] },
      Point { array: [61, -127] },
      Point { array: [85, -85] },
      Point { array: [65, -40] },
      Point { array: [112, -43] },
      Point { array: [89, 96] },
      Point { array: [104, 28] },
      Point { array: [125, 103] },
      Point { array: [82, 123] },
      Point { array: [28, 95] },
      Point { array: [12, 101] },
      Point { array: [-50, 101] },
      Point { array: [-89, 120] },
      Point { array: [-81, 101] },
    ])
    .expect("Invalid Polygon");
    let mut points = polygon.points;
    points.shuffle(&mut rand::thread_rng());

    let res = monotone_polygon::form_monotone_polygon(Vector::from(Point::new([0, 1])), points);

    assert!(res.is_ok());
  }
}
