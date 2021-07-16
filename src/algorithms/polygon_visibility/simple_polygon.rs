//https://link.springer.com/content/pdf/10.1007/BF01937271.pdf

use crate::data::{Point, Polygon};
use crate::{Error, PolygonScalar};

/// Finds the visiblity polygon from a point and a given simple polygon -  O(n)
pub fn get_visibility_polygon_simple<T>(
  point: &Point<T, 2>,
  polygon: &Polygon<T>,
) -> Option<Polygon<T>>
where
  T: PolygonScalar,
{
  Option::None
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
    let naive_polygon = get_visibility_polygon(&point, &polygon);
    let new_polygon = get_visibility_polygon_simple(&point, &polygon);

    match naive_polygon {
      Some(polygon) => {
        prop_assert_eq!(new_polygon.unwrap().points.len(), polygon.points.len())
      }
      None => prop_assert!(new_polygon.is_none()),
    }
  }
}
