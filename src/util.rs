use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

pub fn document() -> web_sys::Document {
    window().document().unwrap()
}

pub fn visible() -> bool {
    document().visibility_state() == web_sys::VisibilityState::Visible
}

pub fn rand_f() -> f64 {
    js_sys::Math::random()
}

/// Random number in 0..n
pub fn rand(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    ((rand_f() * n as f64) as usize).min(n - 1)
}

/// Random number in a..=b
pub fn range(a: i32, b: i32) -> i32 {
    a + rand((b - a + 1).max(1) as usize) as i32
}

pub fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = rand(i + 1);
        v.swap(i, j);
    }
}

pub fn pick<T: Clone>(v: &[T]) -> T {
    v[rand(v.len())].clone()
}

pub fn today() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

/// Date string `days` days before today.
pub fn day_offset(days: u32) -> String {
    let d = js_sys::Date::new_0();
    d.set_date(d.get_date() - days);
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

/// A shared flag that tells running loops/timers the screen is gone.
#[derive(Clone)]
pub struct Alive(Arc<AtomicBool>);

impl Alive {
    pub fn new() -> Self {
        let a = Alive(Arc::new(AtomicBool::new(true)));
        let b = a.clone();
        leptos::prelude::on_cleanup(move || b.0.store(false, Ordering::Relaxed));
        a
    }
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Runs `f(dt_seconds)` every animation frame while `alive`.
pub fn frame_loop(alive: Alive, mut f: impl FnMut(f64) + 'static) {
    type Cb = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
    let cb: Cb = Rc::new(RefCell::new(None));
    let cb2 = cb.clone();
    let mut last: Option<f64> = None;
    *cb2.borrow_mut() = Some(Closure::new(move |t: f64| {
        if !alive.get() {
            return;
        }
        let dt = last.map(|l| ((t - l) / 1000.0).clamp(0.0, 0.05)).unwrap_or(0.0);
        last = Some(t);
        f(dt);
        if let Some(c) = cb.borrow().as_ref() {
            let _ = window().request_animation_frame(c.as_ref().unchecked_ref());
        }
    }));
    let _ = window().request_animation_frame(cb2.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}

/// Resize a canvas to its CSS size × devicePixelRatio and return (ctx, css_w, css_h).
pub fn canvas_ctx(c: &web_sys::HtmlCanvasElement) -> (web_sys::CanvasRenderingContext2d, f64, f64) {
    let dpr = window().device_pixel_ratio().max(1.0);
    let w = c.client_width() as f64;
    let h = c.client_height() as f64;
    let pw = (w * dpr) as u32;
    let ph = (h * dpr) as u32;
    if c.width() != pw || c.height() != ph {
        c.set_width(pw);
        c.set_height(ph);
    }
    let ctx: web_sys::CanvasRenderingContext2d = c.get_context("2d").unwrap().unwrap().unchecked_into();
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    (ctx, w, h)
}
