use gloo_events::EventListener;
use once_cell::sync::Lazy;
use rgeometry_wasm::playground::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, Once};
use wasm_bindgen::{prelude::*, JsCast};

use rgeometry::data::*;

fn ui_range(min: usize, max: usize) {
  let corners = N_CORNERS.load(Ordering::Relaxed);
  let document = web_sys::window().unwrap().document().unwrap();
  let overlay = document.get_element_by_id("ui-overlay").unwrap();
  let val = document
    .create_element("input")
    .unwrap()
    .dyn_into::<web_sys::HtmlInputElement>()
    .unwrap();
  val.set_attribute("type", "range").unwrap();
  val.set_attribute("value", &corners.to_string()).unwrap();
  val.set_attribute("min", &min.to_string()).unwrap();
  val.set_attribute("max", &max.to_string()).unwrap();
  overlay.append_child(&val).unwrap();

  let ev = EventListener::new(&val, "input", move |event| {
    let target = event.target().unwrap();
    let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let n = target.value().parse::<usize>().unwrap_or(3);
    N_CORNERS.store(n, Ordering::Relaxed);
    {
      let mut p = POLYGON.lock().unwrap();
      *p = gen_convex();
    }
    clear_screen();
    demo();
  });
  ev.forget();
}

static N_CORNERS: AtomicUsize = AtomicUsize::new(5);

static POLYGON: Lazy<Mutex<PolygonConvex<Num>>> = Lazy::new(|| Mutex::new(gen_convex()));

fn gen_convex() -> PolygonConvex<Num> {
  let n = N_CORNERS.load(Ordering::Relaxed);
  PolygonConvex::<f32>::random(n, &mut rand::thread_rng()).normalize()
}

fn demo() {
  static START: Once = Once::new();

  START.call_once(|| {
    ui_range(3, 20);
    on_canvas_click(|| {
      {
        let mut p = POLYGON.lock().unwrap();
        *p = gen_convex();
      }
      clear_screen();
      demo();
    });
  });

  let p = POLYGON.lock().unwrap();
  render_polygon(&p);
  set_fill_style("grey");
  fill();

  for pt in p.iter_boundary() {
    render_fixed_point(pt.point());
  }
}

#[wasm_bindgen(start)]
pub fn run() {
  rgeometry_wasm::runner::run(demo);
}
