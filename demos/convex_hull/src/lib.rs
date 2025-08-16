use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::*;

use rgeometry::algorithms::convex_hull;

fn demo() {
  set_viewport(2., 2.);

  let pts = with_points(7);
  if let Ok(convex) = convex_hull(pts.clone()) {
    render_polygon(&convex);
    context().set_fill_style_str("grey");
    context().fill();
  }
  for pt in &pts {
    render_point(&pt);
  }
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
