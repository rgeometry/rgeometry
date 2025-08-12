use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::*;

const N_VERTICES: usize = 7;

fn demo() {
  let p = get_polygon(N_VERTICES);

  render_polygon(&p);
  set_fill_style("grey");
  fill();

  context().save();
  set_line_dash(&[from_pixels(5)]);
  set_stroke_style("lightgrey");
  for (p1, _p2, p3) in p.triangulate().take(N_VERTICES - 3) {
    begin_path();
    move_to_point(&p1);
    line_to_point(&p3);
    stroke();
  }
  context().restore();

  for pt in p.iter_boundary() {
    render_point(&pt);
  }
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
