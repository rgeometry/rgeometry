use crate::data::{Point, Polygon};
use crate::{Error, PolygonScalar};

pub fn new_star_polygon<T>(
  mut vertices: Vec<Point<T, 2>>,
  point: &Point<T, 2>,
) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar,
{
  vertices.sort_by(|a, b| point.ccw_cmp_around(a, b));
  Polygon::new(vertices)
}
