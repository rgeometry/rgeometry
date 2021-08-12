//https://link.springer.com/content/pdf/10.1007/BF01937271.pdf

use std::ops::Index;

use crate::data::polygon::{CursorIter, EdgeIter};
use crate::data::{
  Cursor, DirectedEdge, Direction, EndPoint, HalfLineSoS, IHalfLineLineSegmentSoS,
  ILineLineSegmentSoS, Line, LineSegment, LineSegmentSoS, LineSegmentView, Point, PointId, Polygon,
  Vector,
};
use crate::{Bound, Error, Intersects, Ordering, Orientation, PolygonScalar, SoS};

const TIEBREAKER: SoS = SoS::ClockWise;
struct StartInfo<T> {
  point: Point<T, 2>,
  point_on_polygon: PointId,
}
struct Step<'a, N> {
  process:
    fn(&Point<N, 2>, CursorIter<'a, N>, &mut Vec<Point<N, 2>>, &Point<N, 2>) -> Option<Step<'a, N>>,
  cursor_iter: CursorIter<'a, N>,
  w: Point<N, 2>,
}

/// Finds the visiblity polygon from a point and a given simple polygon -  O(n)
pub fn get_visibility_polygon_simple<T>(
  point: &Point<T, 2>,
  polygon: &Polygon<T>,
) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  let start_point;
  let mut start_cursor;
  match get_start_info(point, polygon) {
    None => panic!("Point is free exterior"),
    Some(info) => {
      start_point = info.point;
      start_cursor = polygon.cursor(info.point_on_polygon);
    }
  }
  // FIXME: coincident start point and start cursor should be handeled in get_start_info() not here
  let mut polygon_points: Vec<Point<T, 2>> = vec![start_point.clone()];
  let mut step;
  if start_point.eq(start_cursor.point()) {
    start_cursor.move_next();
  }

  let cursor_iter = start_cursor.to(Bound::Excluded(start_cursor));

  match Orientation::new(point, &start_point, start_cursor.point()).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      step = {
        polygon_points.push(start_cursor.point().clone());
        Some(Step {
          process: left,
          cursor_iter: cursor_iter,
          w: start_cursor.point().clone(),
        })
      }
    }
    SoS::ClockWise => {
      step = Some(Step {
        process: scan_a,
        cursor_iter: cursor_iter,
        w: start_cursor.point().clone(),
      })
    }
  }

  while let Some(next) = step {
    // FIXME: how to call the function pointer directly?
    let process = next.process;
    step = process(&point, next.cursor_iter, &mut polygon_points, &next.w);
  }

  Some(Polygon::new(polygon_points).expect("Polygon Creation failed"))
}
/// Get the start point (closest point on the positive x axis) and a cursor to the next point
fn get_start_info<T>(view_point: &Point<T, 2>, polygon: &Polygon<T>) -> Option<StartInfo<T>>
where
  T: PolygonScalar,
{
  let x_dir_point = Point::new([T::from_constant(1), T::from_constant(0)]);
  let x_dir = x_dir_point.as_vec();
  let x_ray = HalfLineSoS::new_directed(view_point, &x_dir);
  let mut start_info: Option<StartInfo<T>> = Option::None;

  let mut cursor = polygon.iter_boundary().next().unwrap();
  // FIXME: maybe let directed edge return PointId of the end points?
  for edge in polygon.iter_boundary_edges() {
    //cursor point to edge distination
    cursor.move_next();
    if let Some(_) = x_ray.intersect(edge) {
      // FIXME: problem using Add operator &point + &vector
      let through_point = Point::new([
        view_point.array[0].clone() + x_dir_point.array[0].clone(),
        view_point.array[1].clone() + x_dir_point.array[1].clone(),
      ]);
      let intersection_point = get_intersection(view_point, &through_point, edge.src, edge.dst);
      if check_new_point(view_point, &start_info, &intersection_point) {
        start_info = Some(StartInfo {
          point: intersection_point,
          point_on_polygon: cursor.point_id(),
        });
      }
    }
  }

  return start_info;

  fn check_new_point<T>(
    point: &Point<T, 2>,
    start_info: &Option<StartInfo<T>>,
    curr_point: &Point<T, 2>,
  ) -> bool
  where
    T: PolygonScalar,
  {
    match start_info {
      Some(info) => match point.cmp_distance_to(curr_point, &info.point) {
        Ordering::Less => true,
        _ => false,
      },
      None => true,
    }
  }
}
/// The current segment is taking a left turn
fn left<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  _: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  match cursor_iter.next() {
    None => None,
    Some(curr_cursor) => {
      let nxt_point = curr_cursor.next().point();
      let mut stack_back_iter = polygon_points.iter().rev();
      let p1 = stack_back_iter.next().unwrap();
      let p0 = stack_back_iter.next().unwrap();
      match Orientation::new(point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
        SoS::ClockWise => match Orientation::new(p0, p1, nxt_point).sos(TIEBREAKER) {
          SoS::ClockWise => Some(Step {
            process: scan_a,
            cursor_iter: cursor_iter,
            w: curr_cursor.next().point().clone(),
          }),
          SoS::CounterClockWise => Some(Step {
            process: right,
            cursor_iter: cursor_iter,
            w: curr_cursor.next().point().clone(),
          }),
        },
        SoS::CounterClockWise => {
          polygon_points.push(nxt_point.clone());
          Some(Step {
            process: left,
            cursor_iter: cursor_iter,
            w: curr_cursor.next().point().clone(),
          })
        }
      }
    }
  }
}

/// The current segment is taking a right turn
fn right<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  _: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  let curr_cursor = cursor_iter.next().expect("Exit can be in LEFT or SCANB");
  let nxt_point = curr_cursor.next().point();
  let prev_point = curr_cursor.prev().point();
  //Only check for RA case, we use sos to avoid collinar cases
  let (j, j_prev) = get_ra(point, curr_cursor, polygon_points);
  let intersection = get_intersection(point, curr_cursor.point(), &j_prev, &j);
  polygon_points.push(intersection);

  match Orientation::new(point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
    SoS::CounterClockWise => {
      match Orientation::new(prev_point, curr_cursor.point(), nxt_point).sos(TIEBREAKER) {
        SoS::ClockWise => {
          polygon_points.push(nxt_point.clone());
          Some(Step {
            process: left,
            cursor_iter: cursor_iter,
            w: curr_cursor.next().point().clone(),
          })
        }
        SoS::CounterClockWise => Some(Step {
          process: scan_c,
          cursor_iter: cursor_iter,
          w: curr_cursor.point().clone(),
        }),
      }
    }
    SoS::ClockWise => Some(Step {
      process: right,
      cursor_iter: cursor_iter,
      w: curr_cursor.point().clone(),
    }),
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
    SoS::ClockWise => match Orientation::new(point, p0, curr_cursor.point()).sos(TIEBREAKER) {
      SoS::CounterClockWise => (p1.clone(), p0.clone()),
      SoS::ClockWise => {
        polygon_points.pop();
        get_ra(point, curr_cursor, polygon_points)
      }
    },
  }
}

///SCAN A
fn scan_a<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  _: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  // FIXME: the following block is used in all scan functions, extract
  let s = polygon_points.last().unwrap();
  let ray = HalfLineSoS::new(point, Direction::Through(s));
  let check_intersection = |curr: &Cursor<'_, T>, prev: &Cursor<'_, T>| -> bool {
    let segment = LineSegment::new(
      EndPoint::Inclusive(prev.point().clone()),
      EndPoint::Inclusive(curr.point().clone()),
    );
    match ray.intersect(segment.as_ref()) {
      None => false,
      _ => true,
    }
  };
  //
  cursor_iter.next();
  let c1 = cursor_iter
    .find(|curr| check_intersection(&curr.prev(), curr))
    .expect("There must be an intersection");

  let intersection_point = get_intersection(point, s, c1.prev().point(), c1.point());

  match Orientation::new(point, c1.prev().point(), c1.point()).sos(TIEBREAKER) {
    SoS::ClockWise => match point.cmp_distance_to(s, &intersection_point) {
      Ordering::Greater => Some(Step {
        process: right,
        cursor_iter: cursor_iter,
        w: intersection_point,
      }),
      _ => Some(Step {
        process: scan_d,
        cursor_iter: cursor_iter,
        w: intersection_point,
      }),
    },
    SoS::CounterClockWise => {
      polygon_points.push(intersection_point.clone());
      polygon_points.push(c1.point().clone());
      Some(Step {
        process: left,
        cursor_iter: cursor_iter,
        w: c1.point().clone(),
      })
    }
  }
}

///SCAN B
fn scan_b<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  _: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  let s_t = polygon_points.last().unwrap();
  let s_0 = polygon_points.first().unwrap();
  let sos_segment = LineSegment::new(
    EndPoint::Exclusive(s_t.clone()),
    EndPoint::Inclusive(s_0.clone()),
  );
  let check_intersection = |curr: &Cursor<'_, T>, prev: &Cursor<'_, T>| -> bool {
    let segment = LineSegment::new(
      EndPoint::Inclusive(prev.point().clone()),
      EndPoint::Inclusive(curr.point().clone()),
    );
    match sos_segment.intersect(&segment) {
      None => false,
      _ => true,
    }
  };
  cursor_iter.next();
  let c1 = cursor_iter
    .find(|curr| check_intersection(&curr.prev(), curr))
    .expect("There must be an intersection");

  let intersection_point = get_intersection(s_t, s_0, c1.prev().point(), c1.point());

  match cursor_iter.exhausted {
    true => {
      polygon_points.push(c1.point().clone());
      None // polygon closed
    }
    false => Some(Step {
      process: right,
      cursor_iter: cursor_iter,
      w: intersection_point,
    }),
  }
}

///SCAN C
fn scan_c<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  let s_t = polygon_points.last().unwrap();
  let sos_segment = LineSegment::new(
    EndPoint::Exclusive(s_t.clone()),
    EndPoint::Exclusive(w.clone()),
  );
  let check_intersection = |curr: &Cursor<'_, T>, prev: &Cursor<'_, T>| -> bool {
    let segment = LineSegment::new(
      EndPoint::Inclusive(prev.point().clone()),
      EndPoint::Inclusive(curr.point().clone()),
    );
    match sos_segment.intersect(&segment) {
      None => false,
      _ => true,
    }
  };
  cursor_iter.next();
  let c1 = cursor_iter
    .find(|curr| check_intersection(&curr.prev(), curr))
    .expect("There must be an intersection");
  let intersection_point = get_intersection(s_t, w, c1.prev().point(), c1.point());

  Some(Step {
    process: right,
    cursor_iter: cursor_iter,
    w: intersection_point,
  })
}

///SCAN D
fn scan_d<'a, T>(
  point: &Point<T, 2>,
  mut cursor_iter: CursorIter<'a, T>,
  polygon_points: &mut Vec<Point<T, 2>>,
  w: &Point<T, 2>,
) -> Option<Step<'a, T>>
where
  T: PolygonScalar,
{
  let s_t = polygon_points.last().unwrap();
  let sos_segment = LineSegment::new(
    EndPoint::Exclusive(s_t.clone()),
    EndPoint::Exclusive(w.clone()),
  );
  let check_intersection = |curr: &Cursor<'_, T>, prev: &Cursor<'_, T>| -> bool {
    let segment = LineSegment::new(
      EndPoint::Inclusive(prev.point().clone()),
      EndPoint::Inclusive(curr.point().clone()),
    );
    match sos_segment.intersect(&segment) {
      None => false,
      _ => true,
    }
  };

  cursor_iter.next();
  let c1 = cursor_iter
    .find(|curr| check_intersection(&curr.prev(), curr))
    .expect("There must be an intersection");
  let intersection_point = get_intersection(s_t, w, c1.prev().point(), c1.point());

  Some(Step {
    process: left,
    cursor_iter: cursor_iter,
    w: intersection_point,
  })
}

fn get_intersection<T>(
  r0: &Point<T, 2>,
  r1: &Point<T, 2>,
  s0: &Point<T, 2>,
  s1: &Point<T, 2>,
) -> Point<T, 2>
where
  T: PolygonScalar,
{
  let z_line = Line::new(r0, Direction::Through(r1));
  let j_segment = Line::new(s0, Direction::Through(s1));
  z_line
    .intersection_point(&j_segment)
    .expect("Lines Must Intersect")
}
#[cfg(test)]
mod simple_polygon_testing {
  use crate::algorithms::polygon_visibility::test_polygons;

  use super::super::naive::get_visibility_polygon;
  use super::*;
  use claim::assert_some;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn test_with_naive(polygon: Polygon<i8>, point: Point<i8, 2>) {
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

  #[test]
  fn find_start_point() {
    let testing_info = test_polygons::test_info_1();
    let out_test_point = Point::new([4, 3]);
    let nxt_test_point = Point::new([4, 6]);
    let out_point = get_start_info(&testing_info.point, &testing_info.polygon);
    assert_eq!(assert_some!(&out_point).point, out_test_point);
    assert_eq!(
      testing_info
        .polygon
        .point(assert_some!(&out_point).point_on_polygon),
      &nxt_test_point
    );
  }
}
