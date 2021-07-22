//https://link.springer.com/content/pdf/10.1007/BF01937271.pdf

use crate::data::{Cursor, Direction, EndPoint, Line, LineSegment, Point, Polygon};
use crate::{Bound, Error, Ordering, Orientation, PolygonScalar, SoS};

const TIEBREAKER: SoS = SoS::ClockWise;
struct Process<N>(
  fn(&Point<N, 2>, Cursor<'_, N>, &mut Vec<Point<N, 2>>, &Point<N, 2>) -> Process<N>,
)
where
  N: PolygonScalar;
/// Finds the visiblity polygon from a point and a given simple polygon -  O(n)
pub fn get_visibility_polygon_simple<T>(
  point: &Point<T, 2>,
  polygon: &Polygon<T>,
) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  let mut process;
  let cmp_cursors = |a: &Cursor<'_, T>, b: &Cursor<'_, T>| {
    point.ccw_cmp_around(a, b).then(point.cmp_distance_to(a, b))
  };
  // FIXME: points should start from a point on X axis
  let start_vertex = polygon
    .iter_boundary()
    .min_by(cmp_cursors)
    .expect("polygon must have points");
  let end_vertex = polygon
    .iter_boundary()
    .max_by(cmp_cursors)
    .expect("polygon must have points");

  let mut polygon_points: Vec<Point<T, 2>> = vec![start_vertex.point().clone()];

  match Orientation::new(point, start_vertex.point(), start_vertex.next().point()).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      process = {
        polygon_points.push(start_vertex.next().point().clone());
        Process(left)
      }
    }
    SoS::ClockWise => process = Process(right),
  }

  for vertex in start_vertex.next().to(Bound::Included(end_vertex)) {
    process = process.0(&point, vertex, &mut polygon_points, vertex.next().point());
  }

  Option::None
}

/// The current segment is taking a left turn
fn left<T>(
  point: &Point<T, 2>,
  curr_boundary_point: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  let nxt_point = curr_boundary_point.next().point();
  let mut stack_back_iter = polygon_points.iter().rev();
  let p1 = stack_back_iter.next().unwrap();
  let p0 = stack_back_iter.next().unwrap();
  match Orientation::new(point, curr_boundary_point.point(), nxt_point).sos(TIEBREAKER) {
    SoS::ClockWise => match Orientation::new(p0, p1, nxt_point).sos(TIEBREAKER) {
      SoS::ClockWise => Process(scan_a),
      SoS::CounterClockWise => Process(right),
    },
    SoS::CounterClockWise => {
      polygon_points.push(nxt_point.clone());
      Process(left)
    }
  }
}

/// The current segment is taking a right turn
fn right<T>(
  point: &Point<T, 2>,
  curr_cursor: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  let nxt_point = curr_cursor.next().point();
  let prev_point = curr_cursor.prev().point();
  //Only check for RA case, we use sos to avoid collinar cases
  let (j, j_prev) = get_ra(point, curr_cursor, polygon_points);
  let intersection = get_intersection(point,curr_cursor.point(),&j_prev,&j);
  polygon_points.push(intersection);

  match Orientation::new(point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      match Orientation::new(prev_point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
        SoS::ClockWise => {
          polygon_points.push(nxt_point.clone());
          Process(left)
        }
        SoS::CounterClockWise => Process(scan_c),
      }
    }
    SoS::ClockWise => Process(right),
  }
}

///Case RA for right
fn get_ra<T>(
  point: &Point<T, 2>,
  curr_cursor: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
) -> (Point<T, 2>, Point<T, 2>)
where
  T: PolygonScalar,
{
  let mut stack_back_iter = polygon_points.iter().rev();
  let p1 = stack_back_iter.next().unwrap();
  let p0 = stack_back_iter.next().unwrap();

  match Orientation::new(point, p1, curr_cursor.point()).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      polygon_points.pop();
      get_ra(point, curr_cursor, polygon_points)
    }
    SoS::ClockWise => {
      match Orientation::new(point, p0, curr_cursor.point()).sos(TIEBREAKER) {
        SoS::CounterClockWise => (p1.clone(), p0.clone()),
        SoS::ClockWise => {
          polygon_points.pop();
          get_ra(point, curr_cursor, polygon_points)
        }
      }
    }
  }
}

fn scan_a<T>(
  point: &Point<T, 2>,
  curr_cursor: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  let mut c1 = curr_cursor;
  let mut c0 = curr_cursor.next();
  let s = polygon_points.last().unwrap();
  let check_intersection = |curr:Cursor<'_, T>,prev:Cursor<'_, T> | ->bool {
    Orientation::new(point,s,curr.point()).sos(TIEBREAKER) == SoS::CounterClockWise
    && Orientation::new(point,s,prev.point()).sos(TIEBREAKER) == SoS::ClockWise
  };
  while !check_intersection(c1,c0) {
      c1 = c1.next();
      c0 = c0.next();
  }
  
  let intersection_point = get_intersection(point, s, c0.point(), c1.point());

  match Orientation::new(point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      match Orientation::new(prev_point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
        SoS::ClockWise => {
          polygon_points.push(nxt_point.clone());
          Process(left)
        }
        SoS::CounterClockWise => Process(scan_c),
      }
    }
    SoS::ClockWise => Process(right),
  }


  Process(left)
}
fn scan_b<T>(
  point: &Point<T, 2>,
  curr_boundary_point: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  Process(left)
}
fn scan_c<T>(
  point: &Point<T, 2>,
  curr_boundary_point: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  Process(left)
}
fn scan_d<T>(
  point: &Point<T, 2>,
  curr_boundary_point: Cursor<'_, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Process<T>
where
  T: PolygonScalar,
{
  Process(left)
}

fn get_intersection<T>(l0:&Point<T,2>,l1:&Point<T,2>,s0:&Point<T,2>,s1:&Point<T,2>) -> Point<T,2>
where T: PolygonScalar
{
  let z_line = Line::new(l0, Direction::Through(l1));
  let j_segment = Line::new(s0, Direction::Through(s1));
  z_line
    .intersection_point(&j_segment)
    .expect("Lines Must Intersect")
}
#[cfg(test)]
mod simple_polygon_testing {
  use super::super::naive::get_visibility_polygon;
  use super::*;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn test_no_holes(polygon: Polygon<i8>, point: Point<i8, 2>) {
    // FIXME: get_visibility_polygon overflows
    let cast_polygon: Polygon<i32> = polygon.cast();
    let cast_point: Point<i32, 2> = point.cast();
    let naive_polygon = get_visibility_polygon(&cast_point, &cast_polygon);
    let new_polygon = get_visibility_polygon_simple(&cast_point, &cast_polygon);

    match naive_polygon {
      Some(polygon) => {
        prop_assert_eq!(new_polygon.unwrap().points.len(), polygon.points.len())
      }
      None => prop_assert!(new_polygon.is_none()),
    }
  }
}
