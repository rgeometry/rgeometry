use rgeometry_wasm::playground::*;
use wasm_bindgen::prelude::*;

use rgeometry::algorithms::kernel::naive::kernel;
use rgeometry::data::Polygon;

fn demo() {
  set_viewport(2., 2.);

  let pts = with_points(7);

  // Create a polygon from the points
  if let Ok(poly) = Polygon::new(pts.clone()) {
    // Draw the original polygon in light gray
    render_polygon(&poly);
    context().set_fill_style_str("lightgray");
    context().set_stroke_style_str("gray");
    context().set_line_width(0.01);
    context().fill();
    context().stroke();

    // Compute and draw the kernel
    if let Some(kernel_poly) = kernel(&poly) {
      render_polygon(&kernel_poly);
      context().set_fill_style_str("rgba(100, 150, 255, 0.7)");
      context().set_stroke_style_str("blue");
      context().set_line_width(0.02);
      context().fill();
      context().stroke();
    }
  }

  // Draw the points on top
  for pt in &pts {
    render_point(&pt);
  }
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
