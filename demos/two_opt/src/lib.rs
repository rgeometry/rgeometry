use rgeometry::algorithms::polygonization::*;
use rgeometry::data::*;
use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::wasm_bindgen;

use once_cell::sync::Lazy;
use std::sync::Mutex;

static POLYGON: Lazy<Mutex<Polygon<Num>>> = Lazy::new(|| {
  let pts = get_points(N_VERTICES);
  let p = two_opt_moves(pts, &mut rand::thread_rng()).unwrap();
  Mutex::new(p)
});

const N_VERTICES: usize = 7;

fn demo() {
  set_viewport(2.0, 2.0);

  let pts = get_points(N_VERTICES);

  let mut p = POLYGON.lock().unwrap();
  for (idx, pt) in p.iter_mut().enumerate() {
    *pt = pts[idx].clone();
  }
  resolve_self_intersections(&mut p, &mut rand::thread_rng()).unwrap();
  render_polygon(&p);

  context().set_fill_style(&"grey".into());
  context().fill();

  for pt in &pts {
    render_point(&pt);
  }
}
#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
