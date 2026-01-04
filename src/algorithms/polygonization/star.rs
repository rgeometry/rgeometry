use crate::algorithms::convex_hull::graham_scan::convex_hull;
use crate::data::{Point, PointLocation, Polygon};
use crate::{Error, PolygonScalar, TotalOrd};

/// Creates a star-shaped polygon by sorting vertices around a center point.
///
/// A star-shaped polygon is one where all points on the boundary are visible
/// from the center point. This function creates such a polygon by sorting
/// the vertices counter-clockwise around the given center point.
///
/// # Arguments
///
/// * `vertices` - The vertices of the polygon (at least 3 required)
/// * `point` - The center point around which vertices are sorted
///
/// # Returns
///
/// Returns `Ok(polygon)` if a valid star polygon can be created, or an error if:
/// - There are fewer than 3 vertices
/// - The center point is outside the convex hull of the vertices
/// - All vertices are collinear
///
/// # Example
///
/// ```rust
/// use rgeometry::algorithms::polygonization::new_star_polygon;
/// use rgeometry::data::Point;
///
/// let vertices = vec![
///   Point::new([0, 0]),
///   Point::new([4, 0]),
///   Point::new([4, 4]),
///   Point::new([0, 4]),
/// ];
/// let center = Point::new([2, 2]);
/// let polygon = new_star_polygon(vertices, &center).unwrap();
/// ```
pub fn new_star_polygon<T>(
  mut vertices: Vec<Point<T>>,
  point: &Point<T>,
) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar + TotalOrd,
{
  // Validate that the center point is inside the convex hull of the vertices
  let hull = convex_hull(vertices.clone())?;
  if hull.locate(point) == PointLocation::Outside {
    return Err(Error::ConvexViolation);
  }

  vertices.sort_by(|a, b| point.ccw_cmp_around(a, b));
  Polygon::new(vertices)
}

#[cfg(test)]
mod tests {
  use super::*;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn star_polygon_is_valid(
    #[strategy(proptest::collection::vec(any::<Point<i8, 2>>(), 3..20))] pts: Vec<Point<i8, 2>>,
  ) {
    // Use the centroid as the center point (always inside convex hull if non-degenerate)
    let sum_x: i32 = pts.iter().map(|p| *p.x_coord() as i32).sum();
    let sum_y: i32 = pts.iter().map(|p| *p.y_coord() as i32).sum();
    let n = pts.len() as i32;
    let center = Point::new([(sum_x / n) as i8, (sum_y / n) as i8]);

    if let Ok(p) = new_star_polygon(pts, &center) {
      prop_assert!(p.validate().is_ok());
    }
  }

  #[test]
  fn center_outside_convex_hull_fails() {
    let vertices = vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([4, 4]),
      Point::new([0, 4]),
    ];
    let center = Point::new([10, 10]); // Outside the square
    let result = new_star_polygon(vertices, &center);
    assert!(matches!(result, Err(Error::ConvexViolation)));
  }

  #[test]
  fn center_inside_convex_hull_succeeds() {
    let vertices = vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([4, 4]),
      Point::new([0, 4]),
    ];
    let center = Point::new([2, 2]); // Inside the square
    let result = new_star_polygon(vertices, &center);
    assert!(result.is_ok());
  }
}
