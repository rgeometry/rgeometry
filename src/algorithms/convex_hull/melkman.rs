use crate::data::{Point, Polygon, PolygonConvex};

use crate::PolygonScalar;
use std::collections::VecDeque;

// Wikipedia description of the convex hull problem: https://en.wikipedia.org/wiki/Convex_hull_of_a_simple_polygon
// Melkman's algorithm: https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.512.9681&rep=rep1&type=pdf

// In the right region we pop from the left side, and vice versa.
// In the composition region, which is the region shared by right and left, we pop the deque from both sides.

pub fn convex_hull<T>(polygon: &Polygon<T>) -> PolygonConvex<T>
where
  T: PolygonScalar,
{
  let mut convex_hull: VecDeque<&Point<T, 2>> = VecDeque::new();
  let mut last_idx = 0;
  for cursor in polygon.iter_boundary() {
    let p = cursor.point();
    // Creat a deque with the first 3 points
    if convex_hull.len() < 2 {
      convex_hull.push_back(p);

    // Check for colinear of the first 3 points and remove last verdix if so
    } else if convex_hull.len() == 2 {
      if Point::orient(convex_hull[0], convex_hull[1], p).is_colinear() {
        convex_hull.pop_back();
        convex_hull.push_back(p);
        continue;
      }
      convex_hull.push_front(p);
      convex_hull.push_back(p);

      // Check and correct(if needed) the orientation if the first 3 verdices
      if Point::orient(convex_hull[1], convex_hull[2], convex_hull[3]).is_cw() {
        convex_hull.make_contiguous().reverse();
      }
      last_idx = convex_hull.len() - 1;

    // If the new point is within the polygon, then don't add it to convex hull
    } else if Point::orient(convex_hull[last_idx - 1], convex_hull[last_idx], p).is_ccw()
      && Point::orient(p, convex_hull[1], convex_hull[0]).is_cw()
    {
      continue;
    // Check for wrong rotations/colinear (fron and back) and remove verdices until correct
    } else {
      convex_hull.push_front(p);
      convex_hull.push_back(p);
      last_idx = convex_hull.len() - 1;

      while Point::orient(
        convex_hull[last_idx - 2],
        convex_hull[last_idx - 1],
        convex_hull[last_idx],
      )
      .is_cw()
        || Point::orient(
          convex_hull[last_idx - 2],
          convex_hull[last_idx - 1],
          convex_hull[last_idx],
        )
        .is_colinear()
      {
        convex_hull.remove(last_idx - 1);
        last_idx = convex_hull.len() - 1;
      }

      while Point::orient(convex_hull[2], convex_hull[1], convex_hull[0]).is_ccw()
        || Point::orient(convex_hull[2], convex_hull[1], convex_hull[0]).is_colinear()
      {
        convex_hull.remove(1);
        last_idx = convex_hull.len() - 1;
      }
      if Point::orient(convex_hull[1], convex_hull[0], convex_hull[last_idx - 1]).is_colinear() {
        convex_hull.remove(0);
        last_idx = convex_hull.len() - 1;
      }
    }
  }
  // Pop last duplicated verdix
  convex_hull.pop_back();

  let polygon = Polygon::new_unchecked(convert_deque_to_vec(convex_hull));
  PolygonConvex::new_unchecked(polygon)
}

fn convert_deque_to_vec<T: PolygonScalar>(dque: VecDeque<&Point<T, 2>>) -> Vec<Point<T, 2>> {
  let mut vec: Vec<Point<T, 2>> = Vec::new();
  for i in dque {
    vec.push(i.clone());
  }
  vec
}

#[cfg(test)]
mod tests {
  use crate::testing::{polygon_f64, polygon_ordered};

  use super::*;

  use claims::assert_ok;

  use proptest::proptest as proptest_block;
  use test_strategy::proptest;

  #[test]
  fn unit_test_1() {
    let input = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([2, 2]),
      Point::new([1, 1]),
      Point::new([0, 2]),
    ])
    .unwrap();
    let output = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([2, 2]),
      Point::new([0, 2]),
    ])
    .unwrap();
    // println!("EXPECTED --- {:?}\n", output);
    // println!("GOT --- {:?}\n", convex_hull(&input));
    assert!(Polygon::equals(&convex_hull(&input), &output));
  }

  #[test]
  fn unit_test_2() {
    let input = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([2, 1]),
      Point::new([2, 2]),
      Point::new([1, 2]),
      Point::new([0, 2]),
      Point::new([0, 1]),
    ])
    .unwrap();
    let output = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([2, 2]),
      Point::new([0, 2]),
    ])
    .unwrap();
    // println!("EXPECTED --- {:?}\n", output);
    // println!("GOT --- {:?}\n", convex_hull(&input));
    assert!(Polygon::equals(&convex_hull(&input), &output));
  }

  #[test]
  fn unit_test_3() {
    let input = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([3, 2]),
      Point::new([2, 2]),
      Point::new([-1, 3]),
      Point::new([-1, 2]),
      Point::new([-1, 1]),
      Point::new([0, 2]),
    ])
    .unwrap();
    let output = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([3, 2]),
      Point::new([-1, 3]),
      Point::new([-1, 1]),
    ])
    .unwrap();
    // println!("EXPECTED --- {:?}\n", output);
    // println!("GOT --- {:?}\n", convex_hull(&input));
    assert!(Polygon::equals(&convex_hull(&input), &output));
  }

  #[proptest]
  fn does_not_panic_i8(poly: Polygon<i8>) {
    convex_hull(&poly);
  }

  #[proptest]
  fn does_not_panic_f32(poly: Polygon<f32>) {
    convex_hull(&poly);
  }

  #[proptest]
  fn is_valid_convex_polygon_i8(poly: Polygon<i8>) {
    assert_ok!(convex_hull(&poly).validate());
  }

  #[proptest]
  fn is_valid_convex_polygon_f32(poly: Polygon<f32>) {
    assert_ok!(convex_hull(&poly).validate());
  }

  #[proptest]
  fn is_idempotent_i8(poly: PolygonConvex<i8>) {
    assert!(Polygon::equals(&convex_hull(poly.polygon()), &poly));
  }

  #[track_caller]
  fn graham_scan_eq_prop<T: PolygonScalar>(poly: Polygon<T>) {
    let points: Vec<Point<T>> = poly
      .iter_boundary()
      .map(|cursor| cursor.point())
      .cloned()
      .collect();
    let by_scan = crate::algorithms::convex_hull::graham_scan::convex_hull(points).unwrap();
    assert!(Polygon::equals(&convex_hull(&poly), &by_scan));
  }

  proptest_block! {
    #[test]
    fn graham_scan_eq_prop_i8(poly: Polygon<i8>) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_i16(poly: Polygon<i16>) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_i32(poly: Polygon<i32>) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_i64(poly: Polygon<i64>) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_f32(poly: Polygon<f32>) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_f64(poly in polygon_f64()) {
      graham_scan_eq_prop(poly);
    }

    #[test]
    fn graham_scan_eq_prop_ordered_float(poly in polygon_ordered()) {
      graham_scan_eq_prop(poly);
    }
  }
}
