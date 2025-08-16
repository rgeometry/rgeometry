use rgeometry::algorithms::convex_hull::melkman::*;
use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::wasm_bindgen;

const N_VERTICES: usize = 9;

fn demo() {
  set_viewport(2.0, 2.0);

  let p = get_polygon(N_VERTICES);

  let convex = convex_hull(&p);
  render_polygon(&convex);
  context().set_fill_style_str("lightgrey");
  context().fill();

  render_polygon(&p);
  context().set_fill_style_str("grey");
  context().fill();

  for pt in p.iter_boundary() {
    render_point(&pt.point());
  }
}
#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
