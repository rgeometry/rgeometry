use std::collections::HashSet;

use rgeometry::algorithms::triangulation::constrained_delaunay::constrained_delaunay;
use rgeometry::data::{IndexEdge, PointId, Polygon};
use rgeometry::PolygonScalar;
use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::*;

const N_VERTICES: usize = 9;

fn demo() {
  let polygon = get_polygon(N_VERTICES);
  render_polygon(&polygon);
  set_fill_style("grey");
  fill();

  let diagonals = delaunay_diagonals(&polygon);
  context().save();
  set_stroke_style("#3b82f6");
  set_line_width(from_pixels(2));
  for edge in diagonals {
    begin_path();
    move_to_point(polygon.point(edge.min));
    line_to_point(polygon.point(edge.max));
    stroke();
  }
  context().restore();

  for cursor in polygon.iter_boundary() {
    render_point(&cursor);
  }
}

fn delaunay_diagonals<T>(polygon: &Polygon<T>) -> Vec<IndexEdge>
where
  T: PolygonScalar,
{
  let mut edges = HashSet::new();
  for (a, b, c) in constrained_delaunay(polygon) {
    edges.insert(IndexEdge::new(a, b));
    edges.insert(IndexEdge::new(b, c));
    edges.insert(IndexEdge::new(c, a));
  }

  let boundary: Vec<PointId> = polygon.iter_boundary().map(|cursor| cursor.point_id()).collect();
  if boundary.len() > 1 {
    for window in boundary.windows(2) {
      edges.remove(&IndexEdge::new(window[0], window[1]));
    }
    edges.remove(&IndexEdge::new(boundary[0], *boundary.last().unwrap()));
  }

  edges.into_iter().collect()
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
