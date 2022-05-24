use std::cmp::Ordering;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::panic::Location;

use crate::algorithms::monotone_polygon::new_monotone_polygon;
use crate::data::{
  point, Cursor, DirectedEdge, Direction, EndPoint, HalfLineSoS, IHalfLineLineSegmentSoS, Line,
  LineSegment, LineSegmentView, LineSoS, Point, PointLocation, Polygon, Vector,
};
use crate::{Intersects, Orientation, PolygonScalar, SoS};

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
/// Naive alogrithn for calculating visibility polygon
pub fn get_visibility_polygon<T>(point: &Point<T>, polygon: &Polygon<T>) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  // FIXME: We want to iterate over all vertices, not just boundary vertices.
  let mut vertices: Vec<Cursor<'_, T>> = polygon.iter_boundary().collect();
  vertices.sort_by(|a, b| point.ccw_cmp_around(a, b));

  let mut polygon_points = Vec::new();
  for vertex in vertices {
    let ray_sos = HalfLineSoS::new_through(point, vertex.point());
    let mut right_intersection = NearestIntersection::new(point);
    let mut left_intersection = NearestIntersection::new(point);

    // FIXME: We want to iterate over all edges, not just boundary edges.
    for edge in polygon.iter_boundary_edges() {
      use IHalfLineLineSegmentSoS::*;
      use Orientation::*;
      match ray_sos.intersect(edge) {
        None => (),
        // CoLinear crosing blocks the ray both to the left and to the right
        Some(Crossing(CoLinear)) => {
          let isect = get_intersection(ray_sos, edge);
          left_intersection.push(isect.clone());
          right_intersection.push(isect);
        }
        // Ray blocked on the left side.
        Some(Crossing(CounterClockWise)) => {
          left_intersection.push(get_intersection(ray_sos, edge));
        }
        // Ray blocked on the right side.
        Some(Crossing(ClockWise)) => {
          right_intersection.push(get_intersection(ray_sos, edge));
        }
      }
    }

    match right_intersection.take() {
      Some(interesction) => {
        if point.cmp_distance_to(&interesction, &vertex) != Ordering::Less {
          polygon_points.push(interesction);
        }
      }
      None => return Option::None,
    };
    match left_intersection.take() {
      Some(interesction) => {
        if point.cmp_distance_to(&interesction, &vertex) != Ordering::Less {
          polygon_points.push(interesction);
        }
      }
      None => return Option::None,
    };
  }
  polygon_points.dedup();

  Some(Polygon::new(polygon_points).expect("Polygon Creation failed"))
}

fn get_intersection<T>(sos_line: HalfLineSoS<T>, edge: DirectedEdge<'_, T>) -> Point<T>
where
  T: PolygonScalar,
{
  let segment_line = Line {
    origin: edge.src,
    direction: Direction::Through(edge.dst),
  };
  Line::from(sos_line)
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
    let out_test_points = vec![
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
    let out_test_points = vec![
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
}
