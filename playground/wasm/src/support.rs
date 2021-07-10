#![allow(unused_imports)]
use gloo_events::EventListener;
use wasm_bindgen::prelude::*;

use rgeometry_wasm::playground::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn run() {
  std::panic::set_hook(Box::new(console_error_panic_hook::hook));

  let window = web_sys::window().unwrap();

  let ev = EventListener::new(&window, "resize", move |_event_| {
    redraw();
  });
  ev.forget();

  on_mousemove(|event| {
    upd_mouse(event);
    redraw();
  });
  on_touchmove(|event| {
    upd_touch(event);
    redraw()
  });

  // Defaults
  set_viewport(2.0, 2.0);
  set_font("24px Menlo");
  set_text_align("center");
  set_text_baseline("middle");

  redraw();
}

fn redraw() {
  clear_screen();

  context().save();
  super::main();
  context().restore();
}
