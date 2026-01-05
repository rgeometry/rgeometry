use std::cmp::Ordering;

use crate::data::{
  Cursor, DirectedEdgeView, DirectionView, HalfLineSoS, IHalfLineLineSegmentSoS, LineView, Point,
  Polygon,
};
use crate::{Intersects, Orientation, PolygonScalar};

// Note: The description on wikipedia is quite bad.
// Wikipedia: https://en.wikipedia.org/wiki/Visibility_polygon#Naive_algorithms
// Star polygons: https://en.wikipedia.org/wiki/Star-shaped_polygon
//
// Note about SoS rays (aka HalfLineSoS):
//   SoS rays lean either to the left or to the right. This means they always hit the sides
//   of an edge rather than the vertices. For example:
//
//     a <--- ray
//     |
//     b
//
//   Here we have an edge ('a' to 'b') and a ray that points directly to 'a'. If the ray
//   leans to the left then it will hit to the left of 'a'. That is, it'll hit between
//   'a' and 'b' rather than hitting 'a' directly.
//   If it leans to the right then it'll miss the 'a'-'b' edge altogether.
//   SoS rays are essential for dealing correctly with colinear vertices.
//
//                 a
//   SoS ray --->  |      => Crossing(CoLinear), ray completely blocked.
//                 b
//
//   SoS ray --->  a      => Crossing(ClockWise), ray blocked on the right side.
//                / \
//               b   c
//
//               b   c
//                \ /
//   SoS ray --->  a      => Crossing(CounterClockWise), ray blocked on the left side.
//
//   SoS ray --->  a-b    => None, ray not blocked at all.
//
// Note about visibility polygons:
//   Visibility polygons are always star-shaped: All vertices are visible from the point
//   of origin. This means that we can find the visible vertices in any order we like and
//   then sort them around the point of origin to generate the polygon.
//
// Algorithm overview:
//   1. For each vertex in the polygon (including outer boundary and any holes):
//     1.A Shoot a right-leaning SoS ray towards the vertex and record the intersection
//         point closest to the origin. Note: The SoS ray might not hit the vertex.
//     1.B Shoot a left-leaning SoS ray towards the vertex and record the intersection
//         point closest to the origin. This point may or may not be the same as the point
//         in step 1.A.
//   2. Sort all the intersection points counter-clockwise around the point of origin.
//   3. If two points are at the same angle then sort according to the leaning of the
//      ray that generated the intersection point. Ie. a point from a left-leaning ray
//      should be Ordering::Greater than a point from a right-leaning ray.
//
// Pseudo code:
//   for vertex in poly.vertex {
//     let left_intersections = ... use left-leaning SoS ray ...;
//     let right_intersections = ... use right-leaning SoS ray ...;
//     let left_closest = ...;
//     let right_closest = ...;
//     output.push( (SoS::CounterClockWise, left_closest));
//     output.push( (SoS::ClockWise, right_closest));
//   }
//   output.sort_by(|a,b| origin.ccw_cmp_around(a,b).then(cmp by sos) );
//   Polygon::new(output)
//
// Test cases:
//
//   Input            Output
//
//  /-----\ /----\  /-----\ /----\
//  |     | |    |  |     | |    |
//  |  x  \-/ /--/  |  x  \---\--/
//  |         |     |         |
//  \---------/     \---------/
//
//
//  /-----\ /----\  /-----\ /----\
//  |     | |    |  |     | |    |
//  |  x  \-/    |  |  x  \------\
//  |            |  |            |
//  \------------/  \------------/
//
//  /---------\     /-\     /-\
//  |   /-\   |     |  \   /  |
//  |   \-/   |     |   \ /   |
//  |    x    |     |    x    |
//  \---------/     \---------/
/// Naive algorithm for calculating visibility polygon.
///
/// Given a point inside a simple polygon (without holes), computes the visibility
/// polygon - the region visible from that point.
///
/// # Arguments
///
/// * `point` - The viewpoint from which visibility is computed
/// * `polygon` - A simple polygon (must not contain holes)
///
/// # Returns
///
/// Returns `Some(visibility_polygon)` if the point is inside the polygon,
/// or `None` if the point is outside.
///
/// # Panics
///
/// Panics if the polygon contains holes. This algorithm currently only supports
/// simple polygons without holes.
pub fn get_visibility_polygon<T>(point: &Point<T>, polygon: &Polygon<T>) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  assert!(
    polygon.rings.len() == 1,
    "visibility polygon algorithm does not support polygons with holes"
  );

  let mut vertices: Vec<Cursor<'_, T>> = polygon.iter_boundary().collect();
  vertices.sort_by(|a, b| point.ccw_cmp_around(a, b));

  let mut polygon_points = Vec::new();
  for vertex in vertices {
    let ray_sos = HalfLineSoS::new_through(point, vertex.point());
    let mut right_intersection = NearestIntersection::new(point);
    let mut left_intersection = NearestIntersection::new(point);

    for edge in polygon.iter_boundary_edges() {
      use IHalfLineLineSegmentSoS::*;
      use Orientation::*;
      match ray_sos.intersect(edge) {
        None => (),
        // CoLinear crossing blocks the ray both to the left and to the right
        Some(Crossing(CoLinear)) => {
          let intersection = get_intersection(ray_sos, edge);
          left_intersection.push(intersection.clone());
          right_intersection.push(intersection);
        }
        // Ray blocked on the left side.
        Some(Crossing(CounterClockWise)) => {
          left_intersection.push(get_intersection_colinear(ray_sos, edge));
        }
        // Ray blocked on the right side.
        Some(Crossing(ClockWise)) => {
          right_intersection.push(get_intersection_colinear(ray_sos, edge));
        }
      }
    }

    match right_intersection.take() {
      Some(intersection) => {
        if point.cmp_distance_to(&intersection, &vertex) != Ordering::Less {
          polygon_points.push(intersection);
        }
      }
      None => return None,
    };
    match left_intersection.take() {
      Some(intersection) => {
        if point.cmp_distance_to(&intersection, &vertex) != Ordering::Less {
          polygon_points.push(intersection);
        }
      }
      None => return None,
    };
  }
  polygon_points.dedup();

  Polygon::new(polygon_points).ok()
}

fn get_intersection_colinear<T>(sos_line: HalfLineSoS<T>, edge: DirectedEdgeView<'_, T>) -> Point<T>
where
  T: PolygonScalar,
{
  let line: LineView<T> = sos_line.into();
  match Point::orient_along_direction(line.origin, line.direction, edge.src) {
    Orientation::CoLinear => edge.src.clone(),
    // edge.dst should be colinear with the ray
    _ => edge.dst.clone(),
  }
}

fn get_intersection<T>(sos_line: HalfLineSoS<T>, edge: DirectedEdgeView<'_, T>) -> Point<T>
where
  T: PolygonScalar,
{
  let segment_line = LineView {
    origin: edge.src,
    direction: DirectionView::Through(edge.dst),
  };
  LineView::from(sos_line)
    .intersection_point(&segment_line)
    .expect("LinesMustIntersect")
}

// Container for intersections that only store the nearest point to some origin.
struct NearestIntersection<'a, T> {
  origin: &'a Point<T>,
  nearest_intersection: Option<Point<T>>,
}

impl<'a, T> NearestIntersection<'a, T>
where
  T: PolygonScalar,
{
  fn new(origin: &'a Point<T>) -> NearestIntersection<'a, T> {
    NearestIntersection {
      origin,
      nearest_intersection: None,
    }
  }

  fn push(&mut self, mut intersection: Point<T>) {
    match self.nearest_intersection.as_mut() {
      None => self.nearest_intersection = Some(intersection),
      Some(previous) => {
        if self.origin.cmp_distance_to(&intersection, previous) == Ordering::Less {
          std::mem::swap(previous, &mut intersection);
        }
      }
    }
  }

  fn take(self) -> Option<Point<T>> {
    self.nearest_intersection
  }
}

#[cfg(test)]
mod naive_testing {
  use super::*;

  //     x
  //  /---------\
  //  |         |
  //  |         |
  //  \---------/
  #[test]
  fn point_outside_polygon() {
    let point = Point::new([2, 8]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_polygon = get_visibility_polygon(&point, &input_polygon);

    assert!(out_polygon.is_none())
  }
  //  /-----\ /----\  /-----\ /----\
  //  |     | |    |  |     | |    |
  //  |  x  \-/ /--/  |  x  \---\--/
  //  |         |     |         |
  //  \---------/     \---------/
  #[test]
  fn case_1() {
    let point = Point::new([2, 3]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 3]),
      Point::new([10, 3]),
      Point::new([10, 6]),
      Point::new([6, 6]),
      Point::new([6, 3]),
      Point::new([4, 3]),
      Point::new([4, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_test_points = [
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 3]),
      Point::new([4, 3]),
      Point::new([4, 6]),
      Point::new([0, 6]),
    ];
    let out_polygon = get_visibility_polygon(&point, &input_polygon);
    match out_polygon {
      Some(polygon) => {
        //ToDo: find a better way to compare polygons (maybe pairwise comparison of the points)
        assert_eq!(out_test_points.len(), polygon.points.len())
      }
      None => panic!(),
    }
  }

  //   Input            Output
  //  /-----\ /----\  /-----\ /----\
  //  |     | |    |  |     | |    |
  //  |  x  \-/    |  |  x  \------\
  //  |            |  |            |
  //  \------------/  \------------/
  #[test]
  fn case_2() {
    let point = Point::new([2, 3]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([10, 0]),
      Point::new([10, 6]),
      Point::new([6, 6]),
      Point::new([6, 3]),
      Point::new([4, 3]),
      Point::new([4, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_test_points = [
      Point::new([0, 0]),
      Point::new([10, 0]),
      Point::new([10, 3]),
      Point::new([4, 3]),
      Point::new([4, 6]),
      Point::new([0, 6]),
    ];
    let out_polygon = get_visibility_polygon(&point, &input_polygon);
    match out_polygon {
      Some(polygon) => {
        assert_eq!(out_test_points.len(), polygon.points.len())
      }
      None => panic!(),
    }
  }

  // test with rotating square
  #[test]
  fn test_rotating_square() {
    use std::f64::consts::{FRAC_PI_2, PI};

    fn get_point(theta: f64) -> Point<f64> {
      Point::new([theta.sin(), theta.cos()])
    }

    for i in 0..100 {
      let theta = PI / 2.0 * i as f64 / 100.0;
      let points = vec![
        get_point(theta),
        get_point(theta - FRAC_PI_2),
        get_point(theta - FRAC_PI_2 * 2.0),
        get_point(theta - FRAC_PI_2 * 3.0),
      ];

      let point = Point::new([0.0, 0.0]);
      let polygon = Polygon::new(points.clone()).unwrap();
      let out_polygon = get_visibility_polygon(&point, &polygon).expect("get_visibility_polygon");
      assert_eq!(points, out_polygon.points);
    }
   }

  #[test]
  fn point_on_vertex_returns_none() {
    let point = Point::new([0, 0]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_polygon = get_visibility_polygon(&point, &input_polygon);

    assert!(out_polygon.is_none());
  }

  #[test]
  fn point_on_edge_returns_none() {
    let point = Point::new([4, 0]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_polygon = get_visibility_polygon(&point, &input_polygon);

    assert!(out_polygon.is_none());
  }

  #[test]
  fn point_at_center_of_convex_polygon() {
    let point = Point::new([4, 3]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([8, 6]),
      Point::new([0, 6]),
    ])
    .unwrap();
    let out_polygon = get_visibility_polygon(&point, &input_polygon);

    assert!(out_polygon.is_some());
    if let Some(ref polygon) = out_polygon {
      assert!(!polygon.points.is_empty());
    }
  }

  #[test]
  fn point_on_multiple_edges() {
    let point = Point::new([2, 2]);
    let input_polygon = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([4, 4]),
      Point::new([0, 4]),
    ])
    .unwrap();
    let out_polygon = get_visibility_polygon(&point, &input_polygon);

    assert!(out_polygon.is_some());
  }
}


