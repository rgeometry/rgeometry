use gloo_events::EventListener;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::cast::ToPrimitive;
use rand::distributions::Standard;
use rand::Rng;
use rgeometry::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

fn ui_range(init: usize, min: usize, max: usize) -> Rc<RefCell<usize>> {
  let count = Rc::new(RefCell::new(init));

  let document = web_sys::window().unwrap().document().unwrap();
  let overlay = document.get_element_by_id("ui-overlay").unwrap();
  let val = document
    .create_element("input")
    .unwrap()
    .dyn_into::<web_sys::HtmlInputElement>()
    .unwrap();
  val.set_attribute("type", "range").unwrap();
  val.set_attribute("value", &init.to_string()).unwrap();
  val.set_attribute("min", &min.to_string()).unwrap();
  val.set_attribute("max", &max.to_string()).unwrap();
  overlay.append_child(&val).unwrap();

  let ret = count.clone();
  let ev = EventListener::new(&val, "input", move |event| {
    let target = event.target().unwrap();
    let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
    // let event = event.dyn_ref::<web_sys::InputEvent>().unwrap_throw();
    // log(&event.data().unwrap_or("No data".to_string()));
    let n = target.value().parse::<usize>().unwrap_or(3);
    // log(&target.value());
    *count.borrow_mut() = n;
    main();
  });
  ev.forget();

  ret
}

thread_local! {
  static N_CORNERS: Rc<RefCell<usize>> = ui_range(10, 3, 30);
}

pub fn main() {
  static START: Once = Once::new();

  START.call_once(|| {
    log("This should only happen once.");
  });
  //
  let document = web_sys::window().unwrap().document().unwrap();
  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas
    .dyn_into::<web_sys::HtmlCanvasElement>()
    .map_err(|_| ())
    .unwrap();

  let context = canvas
    .get_context("2d")
    .unwrap()
    .unwrap()
    .dyn_into::<web_sys::CanvasRenderingContext2d>()
    .unwrap();

  context.save();
  context.reset_transform().unwrap();
  context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
  context.restore();

  // context.scale(100., 100.);
  context.set_line_width(3.);
  context.set_line_join("round");

  let mut rng = rand::thread_rng();

  let p: Polygon<BigRational> =
    N_CORNERS.with(|n| ConvexPolygon::random(*n.borrow(), 1000, &mut rng).into());
  let p = p.cast(|v| BigRational::to_f64(&v).unwrap());
  let t = Transform::uniform_scale(300.);
  let p = t * p;

  context.begin_path();
  context.set_line_join("round");
  let mut iter = p.points.iter();
  if let Some(origin) = iter.next() {
    let [x, y] = origin.array;
    context.move_to(x, y);
    while let Some(pt) = iter.next() {
      let [x2, y2] = pt.array;
      context.line_to(x2, y2);
    }
  }
  context.close_path();
  context.stroke();
}

/*


   /---\
  /     \
 /       \
 \       /
  \     /
   \---/

 /-----\
 | /-\ |
 \-/ | |
 /---/ |
 \-----/
*/
