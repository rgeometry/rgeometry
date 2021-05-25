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
use rgeometry::data::*;
use rgeometry::*;
use rgeometry_wasm::playground::*;

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
    on_canvas_click(main);
  });

  set_viewport(2., 2.);

  clear_screen();

  let p: ConvexPolygon<BigRational> =
    N_CORNERS.with(|n| ConvexPolygon::random(*n.borrow(), 1000, &mut rand::thread_rng()));

  render_polygon(&p);
}
