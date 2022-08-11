use crate::data::{Point, Polygon, PolygonConvex};
use crate::PolygonScalar;

// Wikipedia description of the convex hull problem: https://en.wikipedia.org/wiki/Convex_hull_of_a_simple_polygon
// Melkman's algorithm: https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.512.9681&rep=rep1&type=pdf
pub fn convex_hull<T>(_polygon: &Polygon<T>) -> PolygonConvex<T>
where
  T: PolygonScalar,
{
  unimplemented!()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing::*;

  use claim::assert_ok;

  use proptest::collection::*;
  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn does_not_panic(poly: Polygon<i8>) {
    convex_hull(&poly);
  }

  #[proptest]
  fn is_valid_convex_polygon(poly: Polygon<i8>) {
    assert_ok!(convex_hull(&poly).validate());
  }

  #[proptest]
  fn is_idempotent(poly: PolygonConvex<i8>) {
    assert!(convex_hull(poly.polygon()).is(&poly))
  }

  #[proptest]
  fn graham_scan_eq_prop(poly: Polygon<i8>) {
    let points: Vec<Point<i8>> = poly
      .iter_boundary()
      .map(|cursor| cursor.point())
      .cloned()
      .collect();
    let by_scan = crate::algorithms::convex_hull::graham_scan::convex_hull(points).unwrap();
    assert!(convex_hull(&poly).is(&by_scan))
  }
}
