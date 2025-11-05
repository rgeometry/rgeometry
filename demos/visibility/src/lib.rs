use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::*;

use rgeometry::algorithms::visibility::naive::get_visibility_polygon;
use rgeometry::data::Point;

fn demo() {
  set_viewport(2., 2.);

  // Generate a random polygon with movable corners
  let poly = with_polygon(8);

  // Get mouse position (works for both mouse and touch)
  let (mx, my) = mouse_position();
  let mouse_point = Point::new([mx as Num, my as Num]);

  // Render the boundary polygon
  context().set_fill_style_str("lightgrey");
  context().set_stroke_style_str("black");
  render_polygon(&poly);

  // Calculate and render visibility polygon
  if let Some(vis_poly) = get_visibility_polygon(&mouse_point, &poly) {
    context().set_fill_style_str("rgba(100, 150, 255, 0.5)");
    context().set_stroke_style_str("blue");
    render_polygon(&vis_poly);
  }

  // Render the mouse/touch position as a point
  context().set_fill_style_str("red");
  render_point(&mouse_point);
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
