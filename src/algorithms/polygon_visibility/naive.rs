use std::iter::FromIterator;
use std::panic::Location;

use crate::algorithms::monotone_polygon::new_monotone_polygon;
use crate::data::{
  Direction, EndPoint, HalfLineSoS, Line, LineSegment, LineSegmentView, LineSoS, Point,
  PointLocation, Polygon,
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

pub fn get_visibility_polygon<T>(point: &Point<T, 2>, polygon: &Polygon<T>) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  //check whether the point is in the polygon or not
  if polygon.locate(point) == PointLocation::Outside {
    return Option::None;
  }

  let polygon_segments: Vec<LineSegment<T, 2>> = polygon
    .points
    .windows(2)
    .map(|pair| {
      LineSegment::new(
        EndPoint::Inclusive(pair[0].clone()),
        EndPoint::Inclusive(pair[1].clone()),
      )
    })
    .collect();

  let mut polygon_points = Vec::new();
  for vertex in &polygon.points {
    let right_sos = HalfLineSoS {
      line: LineSoS::new_through(point.clone(), vertex.clone()),
      lean: SoS::ClockWise,
    };
    let left_sos = HalfLineSoS {
      line: LineSoS::new_through(point.clone(), vertex.clone()),
      lean: SoS::CounterClockWise,
    };

    let mut right_intersections = Vec::new();
    let mut left_intersections = Vec::new();

    polygon_segments.iter().for_each(|segment| {
      if right_sos.intersect(segment.as_ref()).is_some() {
        right_intersections.push(get_intersection(&right_sos, segment));
      }
      if left_sos.intersect(segment.as_ref()).is_some() {
        left_intersections.push(get_intersection(&left_sos, segment));
      }
    });

    if let Some(point) = right_intersections
      .iter()
      .max_by(|curr, next| point.cmp_distance_to(curr, next))
    {
      polygon_points.push(point.clone());
    };
    if let Some(point) = left_intersections
      .iter()
      .max_by(|curr, next| point.cmp_distance_to(curr, next))
    {
      polygon_points.push(point.clone());
    };
  }
  polygon_points.sort_by(|curr, nxt| point.ccw_cmp_around(curr, nxt));
  polygon_points.dedup();
  Some(Polygon::new(polygon_points).expect("Polygon Creation failed"))
}

fn get_intersection<T>(sos_line: &HalfLineSoS<T, 2>, segment: &LineSegment<T, 2>) -> Point<T, 2>
where
  T: PolygonScalar,
{
  let segment_line = Line {
    origin: segment.min.inner().clone(),
    direction: Direction::Through(segment.max.inner().clone()),
  };
  let sos_line = Line {
    origin: sos_line.line.origin.clone(),
    direction: sos_line.line.direction.clone(),
  };
  sos_line
    .intersection_point(&segment_line)
    .expect("LinesMustIntersect")
}
#[cfg(test)]
mod naive_testing {
  use super::*;

  //   Input            Output
  //  /-----\ /----\  /-----\ /----\
  //  |     | |    |  |     | |    |
  //  |  x  \-/    |  |  x  \------\
  //  |            |  |            |
  //  \------------/  \------------/
  #[test]
  fn case_1() {
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
