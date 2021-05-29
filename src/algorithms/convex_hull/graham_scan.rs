use crate::array::Orientation;
use crate::data::{Point, Polygon, PolygonConvex};
use crate::{Error, PolygonScalar};

// https://en.wikipedia.org/wiki/Graham_scan

// Doesn't allocate.
// Properties:
//    No panics.
//    All Ok results are valid convex polygons.
//    No points are outside the resulting convex polygon.
/// $O(n log n)$ Convex hull of a set of points.
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
  let mut known_good = 1;
  let mut at = 2;
  // Filter out points until all consecutive points are oriented counter-clockwise.
  while at < pts.len() && known_good > 0 {
    let p1 = &pts[at];
    let p2 = &pts[known_good];
    let p3 = &pts[known_good - 1];
    match p3.orientation(p2, p1) {
      Orientation::CounterClockWise => {
        pts.swap(at, known_good + 1);
        at += 1;
        known_good += 1;
      }
      Orientation::ClockWise | Orientation::CoLinear => {
        known_good -= 1;
      }
    }
  }
  pts.truncate(known_good + 1);
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
