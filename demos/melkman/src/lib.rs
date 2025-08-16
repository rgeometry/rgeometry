use rgeometry::algorithms::convex_hull::melkman::*;
use rgeometry::algorithms::polygonization::*;
use rgeometry::data::*;
use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::wasm_bindgen;

use once_cell::sync::Lazy;
use std::sync::Mutex;

const N_VERTICES: usize = 9;

fn demo() {
  set_viewport(2.0, 2.0);

  let p = get_polygon(N_VERTICES);

  let convex = convex_hull(&p);
  render_polygon(&convex);
  context().set_fill_style(&"lightgrey".into());
  context().fill();

  render_polygon(&p);
  context().set_fill_style(&"grey".into());
  context().fill();

  for (nth, pt) in p.iter_boundary().enumerate() {
    render_point(&pt.point());
  }
}
#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
