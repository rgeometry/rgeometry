use crate::algorithms::monotone_polygon::new_monotone_polygon;
use crate::data::{Point, Polygon};
use crate::{Error, PolygonScalar};

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

// pub fn get_visibility_polygon<T>(point: &Point<T, 2>, polygon: &Polygon<T>) -> Result<Polygon<T>,Error>
// where
//   T: PolygonScalar,
// {

// }

#[cfg(test)]
mod naive_testing {
  use super::*;
}
