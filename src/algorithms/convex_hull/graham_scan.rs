use crate::array::Orientation;
use crate::data::{Point, Polygon, PolygonConvex};
use crate::{Error, PolygonScalar};

// https://en.wikipedia.org/wiki/Graham_scan

// Doesn't allocate.
// Properties:
//    No panics.
//    All Ok results are valid convex polygons.
//    No points are outside the resulting convex polygon.
/// $O(n \log n)$ Convex hull of a set of points.
///
/// [Graham scan][wiki] algorithm for finding the smallest convex polygon which
/// contains all the given points.
///
/// # Errors
/// Will return an error iff the input set contains less than three distinct points.
///
/// # Properties
/// * No points from the input set will be outside the returned convex polygon.
/// * All vertices in the convex polygon are from the input set.
///
/// # Examples
///
/// ```rust
/// # pub fn main() {
/// # use rgeometry::algorithms::convex_hull;
/// # use rgeometry::data::Point;
/// # use rgeometry::Error;
/// let empty_set: Vec<Point<i32,2>> = vec![];
/// assert_eq!(
///   convex_hull(empty_set).err(),
///   Some(Error::InsufficientVertices))
/// # }
/// ```
///
/// ```rust
/// # pub fn main() {
/// # use rgeometry::algorithms::convex_hull;
/// # use rgeometry::data::Point;
/// # use rgeometry::Error;
/// let dups = vec![Point::new([0,0])].repeat(3);
/// assert_eq!(
///   convex_hull(dups).err(),
///   Some(Error::InsufficientVertices))
/// # }
/// ```
///
/// ```no_run
/// # pub fn main() {
/// #   use rgeometry::algorithms::convex_hull;
/// #   use rgeometry_wasm::playground::*;
/// #
/// #   static START: std::sync::Once = std::sync::Once::new();
/// #   START.call_once(|| on_mousemove(|_event| main()));
/// #
/// #   clear_screen();
/// #   set_viewport(2., 2.);
/// #
/// #   let canvas = get_canvas();
/// #   let context = get_context_2d(&canvas);
/// #
/// #   let pts = with_points(7);
/// #   let points = pts.clone();
/// if let Ok(convex) = convex_hull(points) {
///   render_polygon(&convex);
/// #   context.set_fill_style(&"grey".into());
/// #   context.fill();
/// }
/// #   for pt in &pts {
/// #     render_point(&pt);
/// #   }
/// # }
/// ```
///
/// <iframe src="https://web.rgeometry.org:20443/loader.html?gist=eac484cd855d001815d23a053919b5ca"></iframe>
///
/// [wiki]: https://en.wikipedia.org/wiki/Graham_scan
pub fn convex_hull<T>(mut pts: Vec<Point<T, 2>>) -> Result<PolygonConvex<T>, Error>
where
  T: PolygonScalar,
  // for<'a> &'a T: PolygonScalarRef<&'a T, T>,
{
  let smallest: &Point<T, 2> = &smallest_point(&pts)?;

  pts.sort_unstable_by(|a, b| {
    smallest
      .ccw_cmp_around(a, b)
      .then_with(|| smallest.cmp_distance_to(a, b))
  });
  pts.dedup();
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  let mut write_idx = 1;
  let mut read_idx = 2;
  // Drop points that are co-linear with our origin.
  {
    let origin = pts[write_idx - 1].clone();
    while read_idx < pts.len() {
      let p2 = &pts[write_idx];
      if origin.orientation(p2, &pts[read_idx]) == Orientation::CoLinear {
        pts.swap(read_idx, write_idx);
        read_idx += 1;
      } else {
        break;
      }
    }
  }
  // Filter out points until all consecutive points are oriented counter-clockwise.
  while read_idx < pts.len() {
    let p1 = &pts[read_idx];
    let p2 = &pts[write_idx];
    let p3 = &pts[write_idx - 1];
    match p3.orientation(p2, p1) {
      Orientation::CounterClockWise => {
        pts.swap(read_idx, write_idx + 1);
        read_idx += 1;
        write_idx += 1;
      }
      Orientation::ClockWise | Orientation::CoLinear => {
        write_idx -= 1;
      }
    }
  }
  pts.truncate(write_idx + 1);
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  Ok(PolygonConvex::new_unchecked(Polygon::new_unchecked(pts)))
}

// Find the smallest point and remove it from the vector
// O(n)
fn smallest_point<T>(pts: &[Point<T, 2>]) -> Result<Point<T, 2>, Error>
where
  T: PolygonScalar,
{
  Ok(
    pts
      .iter()
      .min_by_key(|a| (a.y_coord(), a.x_coord()))
      .ok_or(Error::InsufficientVertices)?
      .clone(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::point::tests::*;
  use crate::data::PointLocation;

  use claim::assert_ok;
  use num_bigint::BigInt;
  use ordered_float::Float;
  use ordered_float::NotNan;
  use std::convert::TryInto;
  use std::fmt::Debug;
  use std::ops::IndexMut;

  use proptest::array::*;
  use proptest::collection::*;
  use proptest::num;
  use proptest::prelude::*;
  use proptest::strategy::*;
  use proptest::test_runner::*;

  #[test]
  fn convex_hull_colinear() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([3, 0]),
      Point::new([4, 0]),
      Point::new([1, 1]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_colinear_rev() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([0, 9]),
      Point::new([0, 8]),
      Point::new([0, 7]),
      Point::new([0, 6]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_invalid() {
    let points: Vec<Point<i64, 2>> = vec![
      Point { array: [0, 0] },
      Point { array: [100, 0] },
      Point { array: [50, 1] },
      Point { array: [40, 1] },
      Point { array: [0, 100] },
    ];
    let points: Vec<Point<BigInt, 2>> =
      points.into_iter().map(|pt| pt.cast(BigInt::from)).collect();
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  proptest! {
    #[test]
    fn convex_hull_prop(pts in vec(any_r(), 0..100)) {
      if let Ok(poly) = convex_hull(pts.clone()) {
        // Prop #1: Results are valid.
        assert_ok!(poly.validate());
        // Prop #2: No points from the input set are outside the polygon.
        for pt in pts.iter() {
          assert_ne!(poly.locate(pt), PointLocation::Outside)
        }
        // Prop #3: All vertices are in the input set.
        for (pt, _meta) in poly.iter() {
          assert!(pts.contains(pt))
        }
      }
    }
  }
}
