use crate::ui::{Finish, Hud, TopBar};
use crate::util::{canvas_ctx, frame_loop, rand_f, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d as Ctx;

const HEARTS: u32 = 5;

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Ready,
    Play,
    Won,
    Lost,
}

struct Rock {
    x: f64,
    y: f64,
    r: f64,
    vx: f64,
    vy: f64,
    hp: i32,
    rot: f64,
    spin: f64,
    hit: f64,
}

struct Part {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    color: &'static str,
}

struct World {
    t: f64,
    ship_x: f64,
    target_x: Option<f64>,
    rocks: Vec<Rock>,
    bullets: Vec<(f64, f64)>,
    stars: Vec<(f64, f64, f64)>,
    parts: Vec<Part>,
    fire_cd: f64,
    spawn_cd: f64,
    star_cd: f64,
    invuln: f64,
    bg: Vec<(f64, f64, f64)>,
    planets: Vec<(f64, f64, f64, &'static str)>,
}

impl World {
    fn new() -> Self {
        World {
            t: 0.0,
            ship_x: -1.0,
            target_x: None,
            rocks: Vec::new(),
            bullets: Vec::new(),
            stars: Vec::new(),
            parts: Vec::new(),
            fire_cd: 0.0,
            spawn_cd: 1.0,
            star_cd: 2.0,
            invuln: 0.0,
            bg: (0..90).map(|_| (rand_f(), rand_f(), 0.5 + rand_f() * 1.6)).collect(),
            planets: vec![
                (0.8, 0.25, 34.0, "#d9825b"),
                (0.15, 0.55, 22.0, "#4aa3d8"),
                (0.65, 0.8, 16.0, "#9b7bd6"),
            ],
        }
    }

    fn burst(&mut self, x: f64, y: f64, color: &'static str, n: usize) {
        for _ in 0..n {
            let a = rand_f() * PI * 2.0;
            let s = 60.0 + rand_f() * 160.0;
            self.parts.push(Part { x, y, vx: a.cos() * s, vy: a.sin() * s, life: 0.6, color });
        }
    }
}

fn circle(ctx: &Ctx, x: f64, y: f64, r: f64, color: &str) {
    ctx.begin_path();
    let _ = ctx.arc(x, y, r.max(0.1), 0.0, PI * 2.0);
    ctx.set_fill_style_str(color);
    ctx.fill();
}

fn draw_ship(ctx: &Ctx, x: f64, y: f64, t: f64) {
    // flame
    let fl = 18.0 + (t * 40.0).sin() * 5.0;
    ctx.begin_path();
    ctx.move_to(x - 9.0, y + 22.0);
    ctx.quadratic_curve_to(x, y + 22.0 + fl * 2.0, x + 9.0, y + 22.0);
    ctx.set_fill_style_str("#ffb02e");
    ctx.fill();
    ctx.begin_path();
    ctx.move_to(x - 5.0, y + 22.0);
    ctx.quadratic_curve_to(x, y + 22.0 + fl * 1.2, x + 5.0, y + 22.0);
    ctx.set_fill_style_str("#fff3a0");
    ctx.fill();
    // fins
    ctx.set_fill_style_str("#e8414a");
    for s in [-1.0, 1.0] {
        ctx.begin_path();
        ctx.move_to(x + s * 12.0, y + 2.0);
        ctx.line_to(x + s * 26.0, y + 26.0);
        ctx.line_to(x + s * 10.0, y + 22.0);
        ctx.close_path();
        ctx.fill();
    }
    // body
    ctx.begin_path();
    let _ = ctx.ellipse(x, y, 16.0, 30.0, 0.0, 0.0, PI * 2.0);
    ctx.set_fill_style_str("#f7f3ea");
    ctx.fill();
    // nose
    ctx.begin_path();
    let _ = ctx.ellipse(x, y - 18.0, 11.0, 13.0, 0.0, PI, PI * 2.0);
    ctx.set_fill_style_str("#e8414a");
    ctx.fill();
    // window with the cat
    circle(ctx, x, y - 2.0, 11.0, "#5aa9e6");
    circle(ctx, x, y - 1.0, 8.0, "#f59a3a");
    ctx.set_fill_style_str("#f59a3a");
    for s in [-1.0, 1.0] {
        ctx.begin_path();
        ctx.move_to(x + s * 7.5, y - 4.0);
        ctx.line_to(x + s * 6.0, y - 11.0);
        ctx.line_to(x + s * 2.5, y - 7.0);
        ctx.fill();
    }
    circle(ctx, x - 3.0, y - 2.0, 1.4, "#3b2a1e");
    circle(ctx, x + 3.0, y - 2.0, 1.4, "#3b2a1e");
}

fn draw_rock(ctx: &Ctx, r: &Rock) {
    ctx.save();
    let _ = ctx.translate(r.x, r.y);
    let _ = ctx.rotate(r.rot);
    ctx.begin_path();
    for i in 0..10 {
        let a = i as f64 / 10.0 * PI * 2.0;
        let k = 0.85 + 0.15 * ((i * 7 % 5) as f64 / 4.0);
        let (px, py) = (a.cos() * r.r * k, a.sin() * r.r * k);
        if i == 0 {
            ctx.move_to(px, py);
        } else {
            ctx.line_to(px, py);
        }
    }
    ctx.close_path();
    ctx.set_fill_style_str(if r.hit > 0.0 { "#e6c9a8" } else if r.hp > 1 { "#8d6e57" } else { "#a88566" });
    ctx.fill();
    circle(ctx, -r.r * 0.3, -r.r * 0.2, r.r * 0.22, "rgba(0,0,0,.18)");
    circle(ctx, r.r * 0.35, r.r * 0.25, r.r * 0.16, "rgba(0,0,0,.18)");
    circle(ctx, r.r * 0.1, r.r * 0.5, r.r * 0.1, "rgba(0,0,0,.15)");
    ctx.restore();
}

#[component]
pub fn Asteroids() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let level = app.level() as f64;
    let duration = 40.0 + level * 5.0;
    let canvas = NodeRef::<leptos::html::Canvas>::new();
    let phase = RwSignal::new(Phase::Ready);
    let hearts = RwSignal::new(HEARTS);
    let progress = RwSignal::new(0.0f64);
    let collected = RwSignal::new(0u32);
    let paused = RwSignal::new(false);
    let restart = RwSignal::new(0u32);

    let world = Rc::new(RefCell::new(World::new()));

    let set_target = {
        let world = world.clone();
        move |e: &web_sys::PointerEvent, c: &web_sys::HtmlCanvasElement| {
            let rect = c.get_bounding_client_rect();
            world.borrow_mut().target_x = Some(e.client_x() as f64 - rect.left());
        }
    };

    {
        let world = world.clone();
        let alive = alive.clone();
        canvas.on_load(move |c| {
            let mut seen_restart = 0;
            frame_loop(alive, move |dt| {
                let (ctx, w, h) = canvas_ctx(&c);
                if w < 10.0 {
                    return;
                }
                let mut wd = world.borrow_mut();
                if restart.get_untracked() != seen_restart {
                    seen_restart = restart.get_untracked();
                    *wd = World::new();
                }
                let ship_y = h - 110.0;
                if wd.ship_x < 0.0 {
                    wd.ship_x = w / 2.0;
                }
                let ph = phase.get_untracked();
                let running = ph == Phase::Play && !paused.get_untracked();
                let dt = if running { dt } else { 0.0 };

                // ---------------------------------------------------- update
                for s in wd.bg.iter_mut() {
                    s.1 += dt * s.2 * 0.02;
                    if s.1 > 1.0 {
                        s.1 -= 1.0;
                    }
                }
                for p in wd.planets.iter_mut() {
                    p.1 += dt * 0.01;
                    if p.1 > 1.15 {
                        p.1 = -0.15;
                        p.0 = rand_f();
                    }
                }
                if running {
                    wd.t += dt;
                    let t = wd.t;
                    if let Some(tx) = wd.target_x {
                        let k = (dt * 12.0).min(1.0);
                        wd.ship_x += (tx.clamp(26.0, w - 26.0) - wd.ship_x) * k;
                    }
                    let sx = wd.ship_x;
                    wd.fire_cd -= dt;
                    if wd.fire_cd <= 0.0 {
                        wd.fire_cd = 0.26;
                        wd.bullets.push((sx, ship_y - 34.0));
                    }
                    wd.spawn_cd -= dt;
                    if wd.spawn_cd <= 0.0 && t < duration - 2.0 {
                        wd.spawn_cd = (1.35 - t * 0.012 - level * 0.08).max(0.45);
                        let r = 16.0 + rand_f() * 20.0;
                        wd.rocks.push(Rock {
                            x: r + rand_f() * (w - 2.0 * r),
                            y: -r,
                            r,
                            vx: (rand_f() - 0.5) * 40.0,
                            vy: 60.0 + level * 10.0 + t * 1.2 + rand_f() * 50.0,
                            hp: if r > 28.0 { 2 } else { 1 },
                            rot: rand_f() * 6.0,
                            spin: (rand_f() - 0.5) * 2.0,
                            hit: 0.0,
                        });
                    }
                    wd.star_cd -= dt;
                    if wd.star_cd <= 0.0 {
                        wd.star_cd = 2.2 + rand_f() * 1.5;
                        wd.stars.push((30.0 + rand_f() * (w - 60.0), -20.0, rand_f() * 6.0));
                    }
                    for b in wd.bullets.iter_mut() {
                        b.1 -= 520.0 * dt;
                    }
                    wd.bullets.retain(|b| b.1 > -20.0);
                    for r in wd.rocks.iter_mut() {
                        r.x += r.vx * dt;
                        r.y += r.vy * dt;
                        r.rot += r.spin * dt;
                        r.hit -= dt;
                        if r.x < r.r || r.x > w - r.r {
                            r.vx = -r.vx;
                        }
                    }
                    for s in wd.stars.iter_mut() {
                        s.1 += 90.0 * dt;
                        s.2 += dt * 4.0;
                    }
                    // bullets vs rocks
                    let mut bursts = Vec::new();
                    let World { rocks, bullets, .. } = &mut *wd;
                    bullets.retain(|b| {
                        for r in rocks.iter_mut() {
                            if r.hp > 0 && (b.0 - r.x).powi(2) + (b.1 - r.y).powi(2) < (r.r + 4.0).powi(2) {
                                r.hp -= 1;
                                r.hit = 0.1;
                                if r.hp == 0 {
                                    bursts.push((r.x, r.y));
                                }
                                return false;
                            }
                        }
                        true
                    });
                    for (x, y) in bursts {
                        audio::pop();
                        wd.burst(x, y, "#c9a27e", 10);
                    }
                    // rocks vs ship
                    wd.invuln -= dt;
                    let mut hit = false;
                    let inv = wd.invuln;
                    for r in wd.rocks.iter_mut() {
                        if r.hp > 0 && (r.x - sx).powi(2) + (r.y - ship_y).powi(2) < (r.r + 18.0).powi(2) {
                            if inv <= 0.0 {
                                hit = true;
                            }
                            r.hp = 0;
                        }
                    }
                    if hit {
                        wd.invuln = 1.5;
                        wd.burst(sx, ship_y, "#ff6b6b", 16);
                        audio::bad();
                        let hv = hearts.get_untracked().saturating_sub(1);
                        hearts.set(hv);
                        if hv == 0 {
                            phase.set(Phase::Lost);
                            audio::say("Oh non ! La fusée est abîmée.");
                        }
                    }
                    wd.rocks.retain(|r| r.hp > 0 && r.y < h + 60.0);
                    // stars vs ship
                    let mut got = 0u32;
                    wd.stars.retain(|s| {
                        if (s.0 - sx).powi(2) + (s.1 - ship_y).powi(2) < 36.0f64.powi(2) {
                            got += 1;
                            false
                        } else {
                            s.1 < h + 30.0
                        }
                    });
                    if got > 0 {
                        audio::good();
                        app.add_stars(got);
                        collected.update(|c| *c += got);
                    }
                    for p in wd.parts.iter_mut() {
                        p.x += p.vx * dt;
                        p.y += p.vy * dt;
                        p.life -= dt;
                    }
                    wd.parts.retain(|p| p.life > 0.0);
                    let pr = (t / duration).min(1.0);
                    if (pr - progress.get_untracked()).abs() > 0.004 {
                        progress.set(pr);
                    }
                    if t >= duration && phase.get_untracked() == Phase::Play {
                        phase.set(Phase::Won);
                        app.add_stars(3);
                    }
                }

                // ---------------------------------------------------- draw
                let g = ctx.create_linear_gradient(0.0, 0.0, 0.0, h);
                let _ = g.add_color_stop(0.0, "#0b1033");
                let _ = g.add_color_stop(1.0, "#2b1d55");
                ctx.set_fill_style_canvas_gradient(&g);
                ctx.fill_rect(0.0, 0.0, w, h);
                for s in &wd.bg {
                    circle(&ctx, s.0 * w, s.1 * h, s.2 * 0.8, "rgba(255,255,255,.7)");
                }
                for p in &wd.planets {
                    circle(&ctx, p.0 * w, p.1 * h, p.2, p.3);
                    circle(&ctx, p.0 * w - p.2 * 0.3, p.1 * h - p.2 * 0.3, p.2 * 0.35, "rgba(255,255,255,.18)");
                }
                for r in &wd.rocks {
                    draw_rock(&ctx, r);
                }
                ctx.set_fill_style_str("#ffd84a");
                for b in &wd.bullets {
                    ctx.fill_rect(b.0 - 2.5, b.1 - 8.0, 5.0, 14.0);
                }
                ctx.set_font("30px sans-serif");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                for s in &wd.stars {
                    let _ = ctx.fill_text("⭐", s.0 + s.2.sin() * 6.0, s.1);
                }
                for p in &wd.parts {
                    ctx.set_global_alpha((p.life / 0.6).clamp(0.0, 1.0));
                    circle(&ctx, p.x, p.y, 3.0, p.color);
                }
                ctx.set_global_alpha(1.0);
                if wd.invuln <= 0.0 || (wd.t * 10.0) as i64 % 2 == 0 {
                    draw_ship(&ctx, wd.ship_x, ship_y, wd.t + dt);
                }
            });
        });
    }

    let on_down = {
        let set_target = set_target.clone();
        move |e: web_sys::PointerEvent| {
            audio::unlock();
            if let Some(c) = canvas.get_untracked() {
                set_target(&e, &c);
            }
            if phase.get_untracked() == Phase::Ready {
                phase.set(Phase::Play);
            }
        }
    };
    let on_move = move |e: web_sys::PointerEvent| {
        if let Some(c) = canvas.get_untracked() {
            set_target(&e, &c);
        }
    };

    let again = Callback::new(move |_| {
        hearts.set(HEARTS);
        progress.set(0.0);
        collected.set(0);
        restart.update(|r| *r += 1);
        phase.set(Phase::Ready);
    });

    view! {
        <div class="screen canvas-screen">
            <canvas node_ref=canvas on:pointerdown=on_down on:pointermove=on_move></canvas>
            <TopBar dark=true>
                <Hud hearts=hearts max=HEARTS progress=progress label="La ceinture d'astéroïdes"/>
                <button class="round pause-btn" on:click=move |_| paused.update(|p| *p = !*p)>{move || if paused.get() { "▶" } else { "II" }}</button>
            </TopBar>
            {move || (phase.get() == Phase::Ready).then(|| view! { <div class="tap-hint">"Glisse ton doigt pour piloter la fusée 🚀"</div> })}
            {move || {
                let ph = phase.get();
                let p = paused.get();
                untrack(move || match ph {
                    Phase::Won => view! {
                        <div class="overlay"><Finish title="Bravo, pilote !" detail=format!("Tu as traversé la ceinture d'astéroïdes et attrapé {} étoiles.", collected.get_untracked()) stars=5 on_again=again/></div>
                    }
                    .into_any(),
                    Phase::Lost => view! {
                        <div class="overlay"><Finish title="Oh non, la fusée est abîmée !" detail="Essaie encore, tu vas y arriver." stars=1 on_again=again/></div>
                    }
                    .into_any(),
                    _ if p => view! {
                        <div class="overlay"><button class="big-btn green" on:click=move |_| paused.set(false)>"▶ Continuer"</button></div>
                    }
                    .into_any(),
                    _ => ().into_any(),
                })
            }}
        </div>
    }
}
