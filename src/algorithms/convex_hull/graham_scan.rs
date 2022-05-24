use crate::data::{Point, Polygon, PolygonConvex};
use crate::{Error, Orientation, PolygonScalar};

// https://en.wikipedia.org/wiki/Graham_scan

// Doesn't allocate.
// Properties:
//    No panics.
//    All Ok results are valid convex polygons.
//    No points are outside the resulting convex polygon.
/// Convex hull of a set of points.
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
/// # Time complexity
/// $O(n \log n)$
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
/// #   let pts = with_points(7);
/// #   let points = pts.clone();
/// if let Ok(convex) = convex_hull(points) {
///   render_polygon(&convex);
/// #   context().set_fill_style(&"grey".into());
/// #   context().fill();
/// }
/// #   for pt in &pts {
/// #     render_point(&pt);
/// #   }
/// # }
/// ```
///
/// <iframe src="https://web.rgeometry.org/wasm/gist/eac484cd855d001815d23a053919b5ca"></iframe>
///
/// [wiki]: https://en.wikipedia.org/wiki/Graham_scan
pub fn convex_hull<T>(mut pts: Vec<Point<T>>) -> Result<PolygonConvex<T>, Error>
where
  T: PolygonScalar,
{
  let smallest = &smallest_point(&pts)?;

  pts.sort_unstable_by(|a, b| {
    smallest
      .ccw_cmp_around(a, b)
      .then_with(|| (a.y_coord(), a.x_coord()).cmp(&(b.y_coord(), b.x_coord())))
    // .then_with(|| smallest.cmp_distance_to(a, b))
  });
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  debug_assert_eq!(&pts[0], smallest);
  let mut write_idx = 1;
  let mut read_idx = 2;
  // Drop points that are co-linear with our origin.
  {
    let origin = pts[write_idx - 1].clone();
    while read_idx < pts.len() {
      if Point::orient(&origin, &pts[write_idx], &pts[read_idx]) == Orientation::CoLinear {
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
    match Point::orient(p3, p2, p1) {
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
fn smallest_point<T>(pts: &[Point<T>]) -> Result<Point<T>, Error>
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
  use crate::data::PointLocation;
  use crate::testing::*;

  use claim::assert_ok;
  use num_bigint::BigInt;

  use proptest::collection::*;
  use proptest::prelude::*;
  use test_strategy::proptest;

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
  fn convex_hull_dups() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 2]),
      Point::new([2, 2]),
      Point::new([5, 1]),
      Point::new([5, 1]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn convex_hull_insufficient_dups() {
    let points = vec![
      Point::new([0, 0]),
      Point::new([0, 0]),
      Point::new([2, 2]),
      Point::new([2, 2]),
      Point::new([0, 0]),
      Point::new([2, 2]),
    ];
    assert_eq!(convex_hull(points).err(), Some(Error::InsufficientVertices));
  }

  #[test]
  fn convex_hull_invalid() {
    let points: Vec<Point<i64>> = vec![
      Point { array: [0, 0] },
      Point { array: [100, 0] },
      Point { array: [50, 1] },
      Point { array: [40, 1] },
      Point { array: [0, 100] },
    ];
    let points: Vec<Point<BigInt, 2>> = points.into_iter().map(|pt| pt.cast()).collect();
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn unit_1() {
    let points: Vec<Point<BigInt>> = vec![
      Point::new([0, 0]).into(),
      Point::new([-1, 1]).into(),
      Point::new([0, 1]).into(),
      Point::new([-717193444810564826, 1]).into(),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[test]
  fn unit_2() {
    let points: Vec<Point<i8>> = vec![
      Point::new([0, 0]),
      Point::new([0, -10]),
      Point::new([-13, 0]),
    ];
    let poly = convex_hull(points).unwrap();
    assert_ok!(poly.validate());
  }

  #[proptest]
  fn convex_hull_prop(#[strategy(vec(any_r(), 0..100))] pts: Vec<Point<BigInt>>) {
    if let Ok(poly) = convex_hull(pts.clone()) {
      // Prop #1: Results are valid.
      prop_assert_eq!(poly.validate().err(), None);
      // Prop #2: No points from the input set are outside the polygon.
      for pt in pts.iter() {
        prop_assert_ne!(poly.locate(pt), PointLocation::Outside)
      }
      // Prop #3: All vertices are in the input set.
      for pt in poly.iter() {
        prop_assert!(pts.contains(pt))
      }
    }
  }

  #[proptest]
  fn convex_hull_prop_i8(#[strategy(vec(any::<Point<i8>>(), 0..100))] pts: Vec<Point<i8>>) {
    if let Ok(poly) = convex_hull(pts.clone()) {
      // Prop #1: Results are valid.
      prop_assert_eq!(poly.validate().err(), None);
      // Prop #2: No points from the input set are outside the polygon.
      for pt in pts.iter() {
        prop_assert_ne!(poly.locate(pt), PointLocation::Outside)
      }
      // Prop #3: All vertices are in the input set.
      for pt in poly.iter() {
        prop_assert!(pts.contains(pt))
      }
    }
  }
}
