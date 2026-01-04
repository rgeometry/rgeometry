use rgeometry::algorithms::polygonization::*;
use rgeometry::data::*;
use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::wasm_bindgen;

fn demo() {
  let mut pts = with_points(9);
  let direction = pts.pop().expect("at least one point");
  let center = Point::new([0.0, 0.0]);

  let vector: Vector<Num, 2> = center - direction;
  if let Ok(p) = new_monotone_polygon(pts.clone(), &vector) {
    set_fill_style("grey");
    render_polygon(&p);
  }

  for pt in pts {
    render_point(&pt);
  }

  render_line(&[center, direction]);
  render_fixed_point(&center);
  render_point(&direction);
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
