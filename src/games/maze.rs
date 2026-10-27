use crate::ui::{Finish, TopBar};
use crate::util::{rand, shuffle, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

const ROUNDS: usize = 3;
// Wall bits
const N: u8 = 1;
const E: u8 = 2;
const S: u8 = 4;
const W: u8 = 8;

const THEMES: [(&str, &str, &str, &str); 3] = [
    // (name, hero, item, goal)
    ("pirates", "🦜", "💎", "⛵"),
    ("chevaliers", "🤴", "🗝️", "🏰"),
    ("dinos", "🦖", "🥚", "🌋"),
];

#[derive(Clone, PartialEq, Debug)]
struct Maze {
    cols: i32,
    rows: i32,
    walls: Vec<u8>,
    start: (i32, i32),
    item: (i32, i32),
    goal: (i32, i32),
    theme: usize,
}

impl Maze {
    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.cols + x) as usize
    }
    fn open(&self, (x, y): (i32, i32), d: (i32, i32)) -> bool {
        let bit = match d {
            (0, -1) => N,
            (1, 0) => E,
            (0, 1) => S,
            _ => W,
        };
        self.walls[self.idx(x, y)] & bit == 0
    }
    fn exits(&self, p: (i32, i32)) -> usize {
        [(0, -1), (1, 0), (0, 1), (-1, 0)].iter().filter(|d| self.open(p, **d)).count()
    }
    fn dist_from(&self, s: (i32, i32)) -> Vec<i32> {
        let mut dist = vec![-1; (self.cols * self.rows) as usize];
        dist[self.idx(s.0, s.1)] = 0;
        let mut q = VecDeque::from([s]);
        while let Some(p) = q.pop_front() {
            for d in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                if self.open(p, d) {
                    let np = (p.0 + d.0, p.1 + d.1);
                    let k = self.idx(np.0, np.1);
                    if dist[k] < 0 {
                        dist[k] = dist[self.idx(p.0, p.1)] + 1;
                        q.push_back(np);
                    }
                }
            }
        }
        dist
    }
}

fn generate(cols: i32, rows: i32, theme: usize) -> Maze {
    let mut walls = vec![N | E | S | W; (cols * rows) as usize];
    let mut seen = vec![false; walls.len()];
    let start = (cols / 2, rows - 1);
    let mut stack = vec![start];
    seen[(start.1 * cols + start.0) as usize] = true;
    while let Some(&(x, y)) = stack.last() {
        let mut dirs = [(0, -1, N, S), (1, 0, E, W), (0, 1, S, N), (-1, 0, W, E)];
        shuffle(&mut dirs);
        let mut moved = false;
        for (dx, dy, bit, back) in dirs {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= cols || ny >= rows {
                continue;
            }
            let k = (ny * cols + nx) as usize;
            if seen[k] {
                continue;
            }
            walls[(y * cols + x) as usize] &= !bit;
            walls[k] &= !back;
            seen[k] = true;
            stack.push((nx, ny));
            moved = true;
            break;
        }
        if !moved {
            stack.pop();
        }
    }
    let mut m = Maze { cols, rows, walls, start, item: start, goal: start, theme };
    // Goal: far from start. Item: somewhere in between, away from both.
    let ds = m.dist_from(start);
    let far = (0..ds.len()).max_by_key(|&k| ds[k]).unwrap();
    m.goal = (far as i32 % cols, far as i32 / cols);
    let dg = m.dist_from(m.goal);
    let maxd = ds[far];
    let mut candidates: Vec<usize> = (0..ds.len())
        .filter(|&k| ds[k] * 3 >= maxd && dg[k] * 3 >= maxd && k != far)
        .collect();
    if candidates.is_empty() {
        candidates = (0..ds.len()).filter(|&k| k != far && ds[k] > 0).collect();
    }
    let it = candidates[rand(candidates.len())];
    m.item = (it as i32 % cols, it as i32 / cols);
    m
}

fn size_for(level: u8) -> (i32, i32) {
    match level {
        1 => (4, 5),
        2 => (5, 6),
        3 => (6, 8),
        4 => (7, 9),
        _ => (8, 11),
    }
}

#[component]
pub fn Maze() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let (cols, rows) = size_for(app.level());
    let first_theme = rand(3);
    let round = RwSignal::new(0usize);
    let maze = RwSignal::new(generate(cols, rows, first_theme));
    let pos = RwSignal::new(maze.get_untracked().start);
    let got_item = RwSignal::new(false);
    let moving = RwSignal::new(false);
    let finished = RwSignal::new(false);

    let new_round = move |r: usize| {
        let m = generate(cols, rows, (first_theme + r) % 3);
        pos.set(m.start);
        maze.set(m);
        got_item.set(false);
        round.set(r);
    };

    // Move in a direction and keep sliding along corridors.
    fn walk(
        d: (i32, i32),
        first: bool,
        alive: Alive,
        maze: RwSignal<Maze>,
        pos: RwSignal<(i32, i32)>,
        got_item: RwSignal<bool>,
        moving: RwSignal<bool>,
        on_goal: Callback<()>,
    ) {
        if !alive.get() {
            return;
        }
        let m = maze.get_untracked();
        let p = pos.get_untracked();
        if !m.open(p, d) {
            if first {
                audio::bump();
            }
            moving.set(false);
            return;
        }
        let np = (p.0 + d.0, p.1 + d.1);
        pos.set(np);
        if np == m.item && !got_item.get_untracked() {
            got_item.set(true);
            audio::good();
            audio::say("Bravo ! Maintenant, va au bout !");
            moving.set(false);
            return;
        }
        if np == m.goal {
            moving.set(false);
            if got_item.get_untracked() {
                on_goal.run(());
            } else {
                audio::say("Il faut d'abord trouver le trésor !");
            }
            return;
        }
        // Keep going while the corridor has exactly two exits and continues straight or turns.
        if m.exits(np) == 2 {
            let back = (-d.0, -d.1);
            let next = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                .into_iter()
                .find(|dd| *dd != back && m.open(np, *dd));
            if let Some(nd) = next {
                set_timeout(
                    move || walk(nd, false, alive, maze, pos, got_item, moving, on_goal),
                    Duration::from_millis(90),
                );
                return;
            }
        }
        moving.set(false);
    }

    let on_goal: Callback<()> = {
        let alive = alive.clone();
        Callback::new(move |_| {
            audio::win();
            app.add_stars(3);
            let alive = alive.clone();
            set_timeout(
                move || {
                    if !alive.get() {
                        return;
                    }
                    let r = round.get_untracked() + 1;
                    if r >= ROUNDS {
                        finished.set(true);
                    } else {
                        new_round(r);
                    }
                },
                Duration::from_millis(900),
            );
        })
    };

    let go = {
        let alive = alive.clone();
        move |d: (i32, i32)| {
            if moving.get_untracked() || finished.get_untracked() {
                return;
            }
            moving.set(true);
            walk(d, true, alive.clone(), maze, pos, got_item, moving, on_goal);
        }
    };

    // Swipe detection on the maze.
    let touch_start = RwSignal::new(None::<(f64, f64)>);
    let go_swipe = go.clone();

    let again = Callback::new(move |_| {
        finished.set(false);
        new_round(0);
    });

    let board = move || {
        let m = maze.get();
        let (hero, item, goal) = (THEMES[m.theme].1, THEMES[m.theme].2, THEMES[m.theme].3);
        let cells = (0..m.cols * m.rows)
            .map(|k| {
                let w = m.walls[k as usize];
                let mut c = String::from("mz");
                for (bit, name) in [(N, " wn"), (E, " we"), (S, " ws"), (W, " ww")] {
                    if w & bit != 0 {
                        c.push_str(name);
                    }
                }
                view! { <div class=c></div> }
            })
            .collect_view();
        let size = format!(
            "width:{}%;height:{}%;font-size:min({}vw, 40px)",
            100.0 / m.cols as f64,
            100.0 / m.rows as f64,
            60.0 / m.cols as f64
        );
        let at = |p: (i32, i32)| format!("translate({}%, {}%)", p.0 * 100, p.1 * 100);
        view! {
            <div
                class="maze"
                style:grid-template-columns=format!("repeat({}, 1fr)", m.cols)
                style:grid-template-rows=format!("repeat({}, 1fr)", m.rows)
                style:aspect-ratio=format!("{} / {}", m.cols, m.rows)
            >
                {cells}
                <div class="sprite" style=format!("{size};left:0;top:0;transform:{}", at(m.goal))>{goal}</div>
                <div class="sprite" style=format!("{size};left:0;top:0;transform:{}", at(m.item)) style:opacity=move || if got_item.get() { "0" } else { "1" }>
                    {item}
                </div>
                <div class="sprite" style=format!("{size};left:0;top:0") style:transform=move || at(pos.get())>{hero}</div>
            </div>
        }
    };

    view! {
        <div class="screen maze-screen">
            <TopBar>
                <div class="maze-target">
                    <span class:got=move || got_item.get()>{move || THEMES[maze.with(|m| m.theme)].2}</span>
                    "›"
                    <span>{move || THEMES[maze.with(|m| m.theme)].3}</span>
                </div>
                <div class="dots">
                    {(0..ROUNDS).map(|i| view! { <div class="dot" class:done=move || i < round.get() class:cur=move || i == round.get()></div> }).collect_view()}
                </div>
            </TopBar>
            {move || {
                if finished.get() {
                    untrack(|| view! { <Finish title="Bravo, explorateur !" detail="Tous les labyrinthes sont réussis." stars=5 on_again=again/> }).into_any()
                } else {
                    let go_swipe = go_swipe.clone();
                    let go = go.clone();
                    let g = move |d: (i32, i32)| {
                        let go = go.clone();
                        move |_| go(d)
                    };
                    view! {
                        <div
                            class="board-wrap"
                            on:pointerdown=move |e: web_sys::PointerEvent| touch_start.set(Some((e.client_x() as f64, e.client_y() as f64)))
                            on:pointerup=move |e: web_sys::PointerEvent| {
                                if let Some((x0, y0)) = touch_start.get_untracked() {
                                    touch_start.set(None);
                                    let (dx, dy) = (e.client_x() as f64 - x0, e.client_y() as f64 - y0);
                                    if dx.abs().max(dy.abs()) > 20.0 {
                                        let d = if dx.abs() > dy.abs() { (dx.signum() as i32, 0) } else { (0, dy.signum() as i32) };
                                        go_swipe(d);
                                    }
                                }
                            }
                        >
                            {board}
                        </div>
                        <div class="pad">
                            <button class="up" on:click=g((0, -1))>"⬆"</button>
                            <button class="left" on:click=g((-1, 0))>"⬅"</button>
                            <button class="down" on:click=g((0, 1))>"⬇"</button>
                            <button class="right" on:click=g((1, 0))>"➡"</button>
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
