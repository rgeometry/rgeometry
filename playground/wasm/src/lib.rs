#![allow(unused_imports)]
use gloo_events::EventListener;
use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;

mod user;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::cast::ToPrimitive;
use rand::distributions::Standard;
use rand::Rng;
use rgeometry::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);

  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() {
  log("New message! 2");
  let window = web_sys::window().unwrap();

  let ev = EventListener::new(&window, "resize", move |_event_| {
    redraw();
  });
  ev.forget();

  redraw();
}

fn redraw() {
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

  context.reset_transform().unwrap();

  context
    .translate(canvas.width() as f64 / 2., canvas.height() as f64 / 2.)
    .unwrap();

  user::main();
}
