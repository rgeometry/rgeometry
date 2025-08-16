use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::*;

static MOUSE_X: AtomicI32 = AtomicI32::new(0);
static MOUSE_Y: AtomicI32 = AtomicI32::new(0);

fn get_mouse() -> (i32, i32) {
  (MOUSE_X.load(Relaxed), MOUSE_Y.load(Relaxed))
}

fn set_mouse(x: i32, y: i32) {
  MOUSE_X.store(x, Relaxed);
  MOUSE_Y.store(y, Relaxed);
}

pub mod playground {
  use rgeometry::algorithms::polygonization::{resolve_self_intersections, two_opt_moves};
  use rgeometry::data::*;

  use gloo_events::{EventListener, EventListenerOptions};
  use rand::distributions::Standard;
  use rand::Rng;
  use std::ops::Deref;
  use std::ops::Index;
  use std::sync::Once;
  use wasm_bindgen::{JsCast, UnwrapThrowExt};
  use web_sys::Path2d;

  use once_cell::sync::Lazy;
  use once_cell::sync::OnceCell;
  use std::sync::Mutex;

  pub type Num = f32;

  pub fn upd_mouse(event: &web_sys::MouseEvent) {
    super::set_mouse(event.offset_x(), event.offset_y())
  }

  pub fn upd_touch(event: &web_sys::TouchEvent) {
    let x = event.touches().get(0).unwrap().client_x();
    let y = event.touches().get(0).unwrap().client_y();
    super::set_mouse(x, y)
  }

  pub fn get_device_pixel_ratio() -> f64 {
    web_sys::window().unwrap().device_pixel_ratio()
  }
  pub fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
  }
  pub fn canvas() -> web_sys::HtmlCanvasElement {
    let canvas = document().get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .map_err(|_| ())
      .unwrap();
    canvas
  }

  pub fn context() -> web_sys::CanvasRenderingContext2d {
    canvas()
      .get_context("2d")
      .unwrap()
      .unwrap()
      .dyn_into::<web_sys::CanvasRenderingContext2d>()
      .unwrap()
  }

  pub fn clear_screen() {
    let canvas = canvas();
    let context = context();
    context.save();
    context.reset_transform().unwrap();
    context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
    context.restore();
  }

  pub fn absolute_mouse_position() -> (i32, i32) {
    super::get_mouse()
  }

  pub fn mouse_position() -> (f64, f64) {
    let (x, y) = absolute_mouse_position();
    inv_canvas_position(x, y)
  }

  pub fn inv_canvas_position(x: i32, y: i32) -> (f64, f64) {
    let ratio = get_device_pixel_ratio();
    let context = context();
    let transform = &context.get_transform().unwrap();
    let inv = transform.inverse();
    let pt = web_sys::DomPointInit::new();
    pt.set_x(x as f64 * ratio);
    pt.set_y(y as f64 * ratio);
    let out = inv.transform_point_with_point(&pt);
    (out.x(), out.y())
  }

  pub fn from_pixels(pixels: u32) -> f64 {
    let (vw, vh) = get_viewport();
    let canvas = canvas();
    let ratio = get_device_pixel_ratio();
    if vw < vh {
      (vw / canvas.width() as f64) * pixels as f64 * ratio
    } else {
      (vh / canvas.height() as f64) * pixels as f64 * ratio
    }
  }
  pub fn get_viewport() -> (f64, f64) {
    let canvas = canvas();
    let context = context();
    let transform = context.get_transform().unwrap();
    let scale = transform.a();
    // let ratio = get_device_pixel_ratio();
    (
      canvas.width() as f64 / scale,
      canvas.height() as f64 / scale,
    )
  }
  pub fn set_viewport(width: f64, height: f64) {
    let pixel_ratio = get_device_pixel_ratio();
    let canvas = canvas();
    let context = context();

    context.reset_transform().unwrap();

    let ratio_width = canvas.width() as f64 / width;
    let ratio_height = canvas.height() as f64 / height;
    let ratio = if ratio_width < ratio_height {
      ratio_width
    } else {
      ratio_height
    };
    context.scale(ratio, -ratio).unwrap();
    context
      .translate(
        canvas.width() as f64 / ratio / 2.,
        -(canvas.height() as f64 / ratio / 2.),
      )
      .unwrap();
    context.set_line_width(2. / ratio * pixel_ratio);
  }

  pub fn render_polygon(poly: &Polygon<Num>) {
    let context = context();

    context.begin_path();
    context.set_line_join("round");
    let mut iter = poly.iter_boundary().map(|pt| pt.point());
    if let Some(origin) = iter.next() {
      let [x, y] = origin.array;
      context.move_to(x.into(), y.into());
      for pt in iter {
        let [x2, y2] = pt.array;
        context.line_to(x2.into(), y2.into());
      }
    }
    context.close_path();
    context.fill();
    context.stroke();
  }

  pub fn render_line(pts: &[Point<Num, 2>]) {
    let context = context();

    context.begin_path();
    context.set_line_join("round");
    let mut iter = pts.iter();
    if let Some(origin) = iter.next() {
      let [x, y] = origin.array;
      context.move_to(x.into(), y.into());
      for pt in iter {
        let [x2, y2] = pt.array;
        context.line_to(x2.into(), y2.into());
      }
    }
    context.stroke();
  }

  pub fn point_path_2d(pt: &Point<Num, 2>, scale: f64) -> Path2d {
    let path = Path2d::new().unwrap();
    path
      .arc(
        (*pt.x_coord()).into(),
        (*pt.y_coord()).into(),
        scale * from_pixels(15), // radius
        0.0,
        std::f64::consts::PI * 2.,
      )
      .unwrap();
    path
  }

  pub fn at_point<F: FnOnce()>(pt: &Point<Num, 2>, cb: F) {
    let context = context();
    context.save();
    context
      .translate((*pt.x_coord()).into(), (*pt.y_coord()).into())
      .unwrap();
    cb();
    context.restore();
  }

  pub fn circle(radius: u32) -> Path2d {
    let path = Path2d::new().unwrap();
    path
      .arc(
        0.0,
        0.0,
        from_pixels(radius), // radius
        0.0,
        std::f64::consts::PI * 2.,
      )
      .unwrap();
    path
  }

  pub fn render_point(pt: &Point<Num, 2>) {
    let path = point_path_2d(pt, 1.0);

    set_fill_style("green");
    fill_with_path_2d(&path);
    stroke_with_path(&path);
  }

  pub fn render_fixed_point(pt: &Point<Num, 2>) {
    let path = point_path_2d(pt, 0.5);

    set_fill_style("grey");
    stroke_with_path(&path);
    fill_with_path_2d(&path);
  }

  pub fn with_points(n: usize) -> Vec<Point<Num, 2>> {
    get_points(n)
  }

  pub fn get_points(n: usize) -> Vec<Point<Num, 2>> {
    static SELECTED: Lazy<Mutex<Option<(usize, i32, i32)>>> = Lazy::new(|| Mutex::new(None));
    static POINTS: Lazy<Mutex<Vec<Point<Num, 2>>>> = Lazy::new(|| Mutex::new(vec![]));

    static START: Once = Once::new();

    START.call_once(|| {
      {
        let mut pts = POINTS.lock().unwrap();
        let mut rng = rand::thread_rng();
        let (width, height) = get_viewport();
        let t = Transform::scale(Vector([(width as Num * 0.8), (height as Num * 0.8)]))
          * Transform::translate(Vector([(-0.5), (-0.5)]));
        while pts.len() < n {
          let pt: Point<Num, 2> = rng.sample(Standard);
          let pt = &t * pt;
          pts.push(pt)
        }
      }

      let handle_select = || {
        let (x, y) = absolute_mouse_position();
        let ratio = get_device_pixel_ratio();
        let context = context();
        let pts = POINTS.lock().unwrap();

        for (i, pt) in pts.deref().iter().enumerate() {
          let path = point_path_2d(pt, 1.0);
          let in_path = context.is_point_in_path_with_path_2d_and_f64(
            &path,
            x as f64 * ratio,
            y as f64 * ratio,
          );
          let in_stroke = context.is_point_in_stroke_with_path_and_x_and_y(
            &path,
            x as f64 * ratio,
            y as f64 * ratio,
          );
          if in_path || in_stroke {
            let mut selected = SELECTED.lock().unwrap();
            *selected = Some((i, x, y));
            break;
          }
        }
      };
      on_mousedown(move |event| {
        upd_mouse(event);
        handle_select();
      });
      on_touchstart(move |event| {
        upd_touch(event);
        handle_select();
      });
      on_mouseup(|_event| *SELECTED.lock().unwrap() = None);
      on_touchend(|_event| *SELECTED.lock().unwrap() = None);
      on_touchmove(move |event| {
        if SELECTED.lock().unwrap().is_some() {
          event.prevent_default();
        }
      });
    });

    // Update points if mouse moved.
    {
      let mut selected = SELECTED.lock().unwrap();

      let (mouse_x, mouse_y) = absolute_mouse_position();
      if let Some((i, x, y)) = *selected {
        let (x, y) = inv_canvas_position(x, y);
        let (ox, oy) = inv_canvas_position(mouse_x, mouse_y);
        let dx = ox - x;
        let dy = oy - y;
        *selected = Some((i, mouse_x, mouse_y));

        let mut pts = POINTS.lock().unwrap();
        let pt = pts.index(i);
        let vector: Vector<Num, 2> = Vector([dx as Num, dy as Num]);
        pts[i] = pt + &vector;
      }
    }

    POINTS.lock().unwrap().clone()
  }

  pub fn with_polygon(n: usize) -> Polygon<Num> {
    get_polygon(n)
  }

  pub fn get_polygon(n: usize) -> Polygon<Num> {
    static POLYGON: OnceCell<Mutex<Polygon<Num>>> = OnceCell::new();
    let mut p = POLYGON
      .get_or_init(|| {
        let pts = with_points(n);
        let p = two_opt_moves(pts, &mut rand::thread_rng()).unwrap();
        Mutex::new(p)
      })
      .lock()
      .unwrap();

    let pts = with_points(n);

    for (idx, pt) in p.iter_mut().enumerate() {
      *pt = pts[idx];
    }
    resolve_self_intersections(&mut p, &mut rand::thread_rng()).unwrap();
    p.clone()
  }

  pub fn on_canvas_click<F>(callback: F)
  where
    F: Fn() + 'static,
  {
    let canvas = super::playground::canvas();
    let listener = EventListener::new(&canvas, "click", move |_event| callback());
    listener.forget();
  }

  pub fn on_mousemove<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::canvas();
    let listener = EventListener::new(&canvas, "mousemove", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }
  pub fn on_mousedown<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::canvas();
    let listener = EventListener::new(&canvas, "mousedown", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }
  pub fn on_mouseup<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::canvas();
    let listener = EventListener::new(&canvas, "mouseup", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchstart<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::canvas();
    let listener = EventListener::new_with_options(&canvas, "touchstart", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchend<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::canvas();
    let listener = EventListener::new_with_options(&canvas, "touchend", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchmove<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::canvas();
    let listener = EventListener::new_with_options(&canvas, "touchmove", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  mod context {
    use super::{context, from_pixels, Num};
    use js_sys::Array;
    use rgeometry::data::*;
    use web_sys::Path2d;

    pub fn set_font(font: &str) {
      context().set_font(font)
    }

    pub fn set_text_align(align: &str) {
      context().set_text_align(align)
    }

    pub fn set_text_baseline(baseline: &str) {
      context().set_text_baseline(baseline)
    }

    pub fn set_fill_style(style: &str) {
      context().set_fill_style_str(style)
    }

    pub fn set_stroke_style(style: &str) {
      context().set_stroke_style_str(style)
    }

    pub fn fill() {
      context().fill()
    }

    pub fn stroke() {
      context().stroke()
    }

    pub fn fill_text(text: &str) {
      context().save();
      let factor = from_pixels(1);
      context().scale(factor, -factor).unwrap();
      context().fill_text(text, 0.0, 0.0).unwrap();
      context().restore();
    }

    pub fn stroke_text(text: &str) {
      context().save();
      let factor = from_pixels(1);
      context().scale(factor, -factor).unwrap();
      context().stroke_text(text, 0.0, 0.0).unwrap();
      context().restore();
    }

    pub fn fill_with_path_2d(path: &Path2d) {
      context().fill_with_path_2d(path)
    }

    pub fn stroke_with_path(path: &Path2d) {
      context().stroke_with_path(path)
    }

    pub fn begin_path() {
      context().begin_path();
    }

    pub fn close_path() {
      context().close_path();
    }

    pub fn set_line_join(join: &str) {
      context().set_line_join(join)
    }

    pub fn move_to(x: Num, y: Num) {
      context().move_to(x.into(), y.into())
    }

    pub fn move_to_point(pt: &Point<Num, 2>) {
      move_to(*pt.x_coord(), *pt.y_coord())
    }

    pub fn line_to(x: Num, y: Num) {
      context().line_to(x.into(), y.into())
    }

    pub fn line_to_point(pt: &Point<Num, 2>) {
      line_to(*pt.x_coord(), *pt.y_coord())
    }

    pub fn set_line_dash(dash: &[f64]) {
      let arr = Array::new();
      for (nth, &dash_len) in dash.iter().enumerate() {
        arr.set(nth as u32, dash_len.into());
      }
      context().set_line_dash(arr.as_ref()).unwrap()
    }
  }
  pub use context::*;
}

pub mod runner {
  use super::playground::*;
  use gloo_events::EventListener;

  pub fn run(demo: fn()) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().unwrap();

    let ev = EventListener::new(&window, "resize", move |_event_| {
      redraw(demo);
    });
    ev.forget();

    on_mousemove(move |event| {
      upd_mouse(event);
      redraw(demo);
    });
    on_touchmove(move |event| {
      upd_touch(event);
      redraw(demo);
    });

    // Defaults
    set_viewport(2.0, 2.0);
    set_font("24px Menlo");
    set_text_align("center");
    set_text_baseline("middle");

    redraw(demo);
  }

  fn redraw(demo: fn()) {
    clear_screen();

    context().save();
    demo();
    context().restore();
  }
}
