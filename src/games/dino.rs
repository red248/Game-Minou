use crate::games::asteroids::Phase;
use crate::ui::{Finish, Hud, TopBar};
use crate::util::{canvas_ctx, frame_loop, rand_f, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d as Ctx;

const HEARTS: u32 = 3;
const GRAVITY: f64 = 1900.0;
const JUMP: f64 = 720.0;

#[derive(Clone, Copy)]
enum Kind {
    Log,
    Rock,
    Bush,
}

struct Obstacle {
    x: f64,
    kind: Kind,
    hit: bool,
}

impl Obstacle {
    fn size(&self) -> (f64, f64) {
        match self.kind {
            Kind::Log => (44.0, 26.0),
            Kind::Rock => (36.0, 32.0),
            Kind::Bush => (30.0, 46.0),
        }
    }
}

struct World {
    t: f64,
    dist: f64,
    y: f64,
    vy: f64,
    jumps: u32,
    obstacles: Vec<Obstacle>,
    apples: Vec<(f64, f64)>,
    next_obstacle: f64,
    next_apple: f64,
    invuln: f64,
    clouds: Vec<(f64, f64, f64)>,
    jump_req: bool,
}

impl World {
    fn new() -> Self {
        World {
            t: 0.0,
            dist: 0.0,
            y: 0.0,
            vy: 0.0,
            jumps: 2,
            obstacles: Vec::new(),
            apples: Vec::new(),
            next_obstacle: 500.0,
            next_apple: 350.0,
            invuln: 0.0,
            clouds: (0..5).map(|i| (i as f64 * 0.25 + rand_f() * 0.1, 0.12 + rand_f() * 0.25, 0.6 + rand_f() * 0.6)).collect(),
            jump_req: false,
        }
    }
}

fn circle(ctx: &Ctx, x: f64, y: f64, r: f64, color: &str) {
    ctx.begin_path();
    let _ = ctx.arc(x, y, r.max(0.1), 0.0, PI * 2.0);
    ctx.set_fill_style_str(color);
    ctx.fill();
}

fn ellipse(ctx: &Ctx, x: f64, y: f64, rx: f64, ry: f64, rot: f64, color: &str) {
    ctx.begin_path();
    let _ = ctx.ellipse(x, y, rx, ry, rot, 0.0, PI * 2.0);
    ctx.set_fill_style_str(color);
    ctx.fill();
}

/// Cute green dino; (x, y) = feet position.
fn draw_dino(ctx: &Ctx, x: f64, y: f64, t: f64, on_ground: bool) {
    let green = "#5cc46a";
    let dark = "#3f9a4e";
    // tail
    ctx.begin_path();
    ctx.move_to(x - 14.0, y - 30.0);
    ctx.quadratic_curve_to(x - 40.0, y - 30.0, x - 46.0, y - 44.0);
    ctx.quadratic_curve_to(x - 30.0, y - 18.0, x - 8.0, y - 18.0);
    ctx.set_fill_style_str(green);
    ctx.fill();
    // legs
    let step = if on_ground { (t * 18.0).sin() * 6.0 } else { 3.0 };
    ctx.set_fill_style_str(dark);
    ctx.fill_rect(x - 12.0 + step, y - 16.0, 9.0, 16.0);
    ctx.fill_rect(x + 4.0 - step, y - 16.0, 9.0, 16.0);
    // spikes
    ctx.set_fill_style_str("#f4b63a");
    for i in 0..4 {
        let sx = x - 18.0 + i as f64 * 9.0;
        ctx.begin_path();
        ctx.move_to(sx - 5.0, y - 40.0 - (i as f64 * 2.0));
        ctx.line_to(sx, y - 50.0 - (i as f64 * 2.0));
        ctx.line_to(sx + 5.0, y - 40.0 - (i as f64 * 2.0));
        ctx.fill();
    }
    // body + belly
    ellipse(ctx, x, y - 28.0, 22.0, 17.0, 0.0, green);
    ellipse(ctx, x + 6.0, y - 24.0, 12.0, 10.0, 0.0, "#c9f0b8");
    // head
    ellipse(ctx, x + 22.0, y - 48.0, 17.0, 14.0, -0.15, green);
    circle(ctx, x + 27.0, y - 52.0, 5.0, "white");
    circle(ctx, x + 28.5, y - 52.0, 2.6, "#2b2b2b");
    circle(ctx, x + 18.0, y - 43.0, 3.0, "rgba(255,120,140,.6)");
    ctx.begin_path();
    let _ = ctx.arc(x + 31.0, y - 44.0, 4.0, 0.2, PI - 0.2);
    ctx.set_stroke_style_str("#2b5a32");
    ctx.set_line_width(2.0);
    ctx.stroke();
    // little arm
    ellipse(ctx, x + 16.0, y - 28.0, 5.0, 3.0, 0.6, dark);
}

fn draw_obstacle(ctx: &Ctx, o: &Obstacle, ground: f64) {
    let (w, h) = o.size();
    match o.kind {
        Kind::Log => {
            ctx.set_fill_style_str("#9a6233");
            ctx.fill_rect(o.x - w / 2.0, ground - h, w, h);
            ellipse(ctx, o.x + w / 2.0, ground - h / 2.0, 7.0, h / 2.0, 0.0, "#e3b27a");
            ellipse(ctx, o.x + w / 2.0, ground - h / 2.0, 3.0, h / 4.0, 0.0, "#b97c45");
        }
        Kind::Rock => {
            ellipse(ctx, o.x, ground - h / 2.0 + 2.0, w / 2.0, h / 2.0, 0.0, "#9aa0a8");
            circle(ctx, o.x - 6.0, ground - h * 0.6, 4.0, "#b9bec5");
        }
        Kind::Bush => {
            ctx.set_fill_style_str("#2f9e5b");
            ctx.fill_rect(o.x - 6.0, ground - h, 12.0, h);
            ctx.fill_rect(o.x - 16.0, ground - h * 0.7, 10.0, 6.0);
            ctx.fill_rect(o.x - 16.0, ground - h * 0.95, 6.0, h * 0.3);
            ctx.fill_rect(o.x + 6.0, ground - h * 0.55, 10.0, 6.0);
            ctx.fill_rect(o.x + 10.0, ground - h * 0.8, 6.0, h * 0.3);
        }
    }
}

#[component]
pub fn Dino() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let level = app.level() as f64;
    let duration = 35.0 + level * 5.0;
    let canvas = NodeRef::<leptos::html::Canvas>::new();
    let phase = RwSignal::new(Phase::Ready);
    let hearts = RwSignal::new(HEARTS);
    let progress = RwSignal::new(0.0f64);
    let apples = RwSignal::new(0u32);
    let paused = RwSignal::new(false);
    let restart = RwSignal::new(0u32);
    let world = Rc::new(RefCell::new(World::new()));

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
                let ground = h * 0.8;
                let dx = w * 0.24;
                let running = phase.get_untracked() == Phase::Play && !paused.get_untracked();
                let dt = if running { dt } else { 0.0 };
                let speed = 230.0 + level * 15.0 + wd.t * 2.5;

                // ---------------------------------------------------- update
                if running {
                    wd.t += dt;
                    wd.dist += speed * dt;
                    if wd.jump_req {
                        wd.jump_req = false;
                        if wd.jumps > 0 {
                            wd.vy = if wd.jumps == 2 { JUMP } else { JUMP * 0.8 };
                            wd.jumps -= 1;
                            audio::pop();
                        }
                    }
                    wd.vy -= GRAVITY * dt;
                    wd.y += wd.vy * dt;
                    if wd.y <= 0.0 {
                        wd.y = 0.0;
                        wd.vy = 0.0;
                        wd.jumps = 2;
                    }
                    for o in wd.obstacles.iter_mut() {
                        o.x -= speed * dt;
                    }
                    for a in wd.apples.iter_mut() {
                        a.0 -= speed * dt;
                    }
                    for c in wd.clouds.iter_mut() {
                        c.0 -= dt * 0.02 * c.2;
                        if c.0 < -0.2 {
                            c.0 = 1.2;
                        }
                    }
                    wd.next_obstacle -= speed * dt;
                    if wd.next_obstacle <= 0.0 && wd.t < duration - 2.5 {
                        let kind = match (rand_f() * 3.0) as u32 {
                            0 => Kind::Log,
                            1 => Kind::Rock,
                            _ => Kind::Bush,
                        };
                        wd.obstacles.push(Obstacle { x: w + 40.0, kind, hit: false });
                        wd.next_obstacle = 300.0 + rand_f() * 320.0 - level * 15.0 + speed * 0.4;
                    }
                    wd.next_apple -= speed * dt;
                    if wd.next_apple <= 0.0 && wd.t < duration - 2.0 {
                        wd.apples.push((w + 40.0, 50.0 + rand_f() * 110.0));
                        wd.next_apple = 380.0 + rand_f() * 500.0;
                    }
                    // collisions
                    wd.invuln -= dt;
                    let (dy, inv) = (wd.y, wd.invuln);
                    let mut hit = false;
                    for o in wd.obstacles.iter_mut() {
                        let (ow, oh) = o.size();
                        let overlap_x = (o.x - dx).abs() < ow / 2.0 + 14.0;
                        let overlap_y = dy < oh - 6.0;
                        if !o.hit && overlap_x && overlap_y && inv <= 0.0 {
                            o.hit = true;
                            hit = true;
                        }
                    }
                    if hit {
                        wd.invuln = 1.3;
                        audio::bad();
                        let hv = hearts.get_untracked().saturating_sub(1);
                        hearts.set(hv);
                        if hv == 0 {
                            phase.set(Phase::Lost);
                            audio::say("Aïe ! Le dino est fatigué.");
                        }
                    }
                    let mut got = 0;
                    wd.apples.retain(|a| {
                        let near = (a.0 - dx).abs() < 30.0 && (ground - a.1 - (ground - dy - 30.0)).abs() < 40.0;
                        if near {
                            got += 1;
                        }
                        !near && a.0 > -40.0
                    });
                    if got > 0 {
                        audio::good();
                        app.add_stars(got);
                        apples.update(|a| *a += got);
                    }
                    wd.obstacles.retain(|o| o.x > -60.0);
                    let pr = (wd.t / duration).min(1.0);
                    if (pr - progress.get_untracked()).abs() > 0.004 {
                        progress.set(pr);
                    }
                    if wd.t >= duration && phase.get_untracked() == Phase::Play {
                        phase.set(Phase::Won);
                        app.add_stars(3);
                    }
                }

                // ---------------------------------------------------- draw
                let g = ctx.create_linear_gradient(0.0, 0.0, 0.0, ground);
                let _ = g.add_color_stop(0.0, "#6ab8f0");
                let _ = g.add_color_stop(1.0, "#d6f0ff");
                ctx.set_fill_style_canvas_gradient(&g);
                ctx.fill_rect(0.0, 0.0, w, h);
                circle(&ctx, w * 0.78, h * 0.2, 34.0, "#ffc93c");
                circle(&ctx, w * 0.78, h * 0.2, 46.0, "rgba(255,201,60,.25)");
                for c in &wd.clouds {
                    let (cx, cy, s) = (c.0 * w, c.1 * h, c.2);
                    circle(&ctx, cx, cy, 22.0 * s, "white");
                    circle(&ctx, cx + 22.0 * s, cy + 4.0, 17.0 * s, "white");
                    circle(&ctx, cx - 22.0 * s, cy + 6.0, 15.0 * s, "white");
                }
                // hills (parallax)
                for (k, color, amp, base) in [(0.3, "#a8dca0", 40.0, 0.62), (0.6, "#8acb7f", 26.0, 0.7)] {
                    ctx.begin_path();
                    ctx.move_to(0.0, ground);
                    let mut x = 0.0;
                    while x <= w + 10.0 {
                        let v = ((x + wd.dist * k) * 0.008).sin() * amp + ((x + wd.dist * k) * 0.019).sin() * amp * 0.4;
                        ctx.line_to(x, h * base - v);
                        x += 10.0;
                    }
                    ctx.line_to(w, ground);
                    ctx.close_path();
                    ctx.set_fill_style_str(color);
                    ctx.fill();
                }
                // ground
                ctx.set_fill_style_str("#8a5a32");
                ctx.fill_rect(0.0, ground, w, h - ground);
                ctx.set_fill_style_str("#5fb35a");
                ctx.fill_rect(0.0, ground, w, 8.0);
                ctx.set_fill_style_str("#74482a");
                let off = wd.dist % 60.0;
                let mut x = -off;
                while x < w {
                    ctx.fill_rect(x, ground + 24.0, 18.0, 4.0);
                    x += 60.0;
                }
                for o in &wd.obstacles {
                    draw_obstacle(&ctx, o, ground);
                }
                ctx.set_font("30px sans-serif");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                for a in &wd.apples {
                    let _ = ctx.fill_text("🍎", a.0, ground - a.1 + (wd.t * 4.0 + a.0 * 0.05).sin() * 4.0);
                }
                if wd.invuln <= 0.0 || (wd.t * 10.0) as i64 % 2 == 0 {
                    draw_dino(&ctx, dx, ground - wd.y, wd.t, wd.y <= 0.0);
                }
                // finish flag near the end
                let remain = (duration - wd.t) * speed;
                if remain < w {
                    let fx = dx + remain;
                    ctx.set_fill_style_str("#6b4a2b");
                    ctx.fill_rect(fx, ground - 90.0, 5.0, 90.0);
                    ctx.set_fill_style_str("#f5578f");
                    ctx.begin_path();
                    ctx.move_to(fx + 5.0, ground - 90.0);
                    ctx.line_to(fx + 45.0, ground - 76.0);
                    ctx.line_to(fx + 5.0, ground - 62.0);
                    ctx.fill();
                }
            });
        });
    }

    let on_down = {
        let world = world.clone();
        move |_e: web_sys::PointerEvent| {
            audio::unlock();
            match phase.get_untracked() {
                Phase::Ready => phase.set(Phase::Play),
                Phase::Play => world.borrow_mut().jump_req = true,
                _ => {}
            }
        }
    };

    let again = Callback::new(move |_| {
        hearts.set(HEARTS);
        progress.set(0.0);
        apples.set(0);
        restart.update(|r| *r += 1);
        phase.set(Phase::Ready);
    });

    view! {
        <div class="screen canvas-screen">
            <canvas node_ref=canvas on:pointerdown=on_down></canvas>
            <TopBar>
                <Hud hearts=hearts max=HEARTS progress=progress label="La vallée"/>
                <button class="round pause-btn" on:click=move |_| paused.update(|p| *p = !*p)>{move || if paused.get() { "▶" } else { "II" }}</button>
            </TopBar>
            {move || (phase.get() == Phase::Ready).then(|| view! { <div class="tap-hint">"Touche l'écran pour sauter ! (2 fois = double saut)"</div> })}
            {move || {
                let ph = phase.get();
                let p = paused.get();
                untrack(move || match ph {
                    Phase::Won => view! {
                        <div class="overlay"><Finish title="Bravo, le dino est arrivé !" detail=format!("Tu as ramassé {} pommes.", apples.get_untracked()) stars=5 on_again=again/></div>
                    }
                    .into_any(),
                    Phase::Lost => view! {
                        <div class="overlay"><Finish title="Oh non ! Le dino est fatigué." detail="Essaie encore !" stars=1 on_again=again/></div>
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
