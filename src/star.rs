use crate::data::{
  Point, 
  Polygon
};
use crate::{PolygonScalar};

pub fn points_to_starshaped_polygon<T>(mut vertices: Vec<Point<T, 2>>, point: &Point<T, 2>) -> Polygon<T>
where
  T: PolygonScalar,
{  
  vertices.sort_by(|a, b| point.ccw_cmp_around(a, b));
  Polygon::new_unchecked(vertices)
}