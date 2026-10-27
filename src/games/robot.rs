use crate::ui::{Cat, Finish, TopBar};
use crate::util::{rand, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

const PUZZLES: usize = 5;
const SLOTS: usize = 10;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Dir {
    Left,
    Up,
    Down,
    Right,
}

impl Dir {
    fn delta(self) -> (i32, i32) {
        match self {
            Dir::Left => (-1, 0),
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Right => (1, 0),
        }
    }
    fn arrow(self) -> &'static str {
        match self {
            Dir::Left => "⬅",
            Dir::Up => "⬆",
            Dir::Down => "⬇",
            Dir::Right => "➡",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Dir::Left => "c-left",
            Dir::Up => "c-up",
            Dir::Down => "c-down",
            Dir::Right => "c-right",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Puzzle {
    n: i32,
    start: (i32, i32),
    plant: (i32, i32),
    rocks: Vec<(i32, i32)>,
}

fn shortest(p: &Puzzle) -> Option<usize> {
    let n = p.n;
    let mut seen = vec![false; (n * n) as usize];
    let mut q = VecDeque::from([(p.start, 0usize)]);
    seen[(p.start.1 * n + p.start.0) as usize] = true;
    while let Some(((x, y), d)) = q.pop_front() {
        if (x, y) == p.plant {
            return Some(d);
        }
        for dir in [Dir::Left, Dir::Up, Dir::Down, Dir::Right] {
            let (dx, dy) = dir.delta();
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= n || ny >= n || p.rocks.contains(&(nx, ny)) {
                continue;
            }
            let k = (ny * n + nx) as usize;
            if !seen[k] {
                seen[k] = true;
                q.push_back(((nx, ny), d + 1));
            }
        }
    }
    None
}

fn make_puzzle(level: u8, round: usize) -> Puzzle {
    let n = if level <= 2 { 4 } else { 5 };
    let (min_d, max_d) = match level {
        1 => (1, 3),
        2 => (2, 5),
        3 => (3, 6),
        _ => (4, 9),
    };
    let rocks_n = match level {
        1 | 2 => 0,
        3 => 1 + round % 2,
        _ => 2 + rand(3),
    };
    loop {
        let cell = || (rand(n as usize) as i32, rand(n as usize) as i32);
        let start = cell();
        let plant = cell();
        let mut rocks = Vec::new();
        while rocks.len() < rocks_n {
            let r = cell();
            if r != start && r != plant && !rocks.contains(&r) {
                rocks.push(r);
            }
        }
        let p = Puzzle { n, start, plant, rocks };
        if let Some(d) = shortest(&p) {
            if d >= min_d && d <= max_d {
                return p;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Edit,
    Running,
    Won,
}

#[component]
pub fn Robot() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let level = app.level();
    let round = RwSignal::new(0usize);
    let puzzle = RwSignal::new(make_puzzle(level, 0));
    let prog = RwSignal::new(Vec::<Dir>::new());
    let pos = RwSignal::new(puzzle.get_untracked().start);
    let phase = RwSignal::new(Phase::Edit);
    let step = RwSignal::new(None::<usize>);
    let bonk = RwSignal::new(false);
    let msg = RwSignal::new("Aide le robot à arroser la plante !".to_string());
    let finished = RwSignal::new(false);

    let reset_robot = move || {
        pos.set(puzzle.get_untracked().start);
        step.set(None);
        phase.set(Phase::Edit);
    };

    let add = move |d: Dir| {
        if phase.get_untracked() != Phase::Edit {
            return;
        }
        if pos.get_untracked() != puzzle.get_untracked().start {
            pos.set(puzzle.get_untracked().start);
        }
        if prog.with_untracked(|p| p.len()) < SLOTS {
            audio::pop();
            prog.update(|p| p.push(d));
        }
    };

    // Runs the program one step at a time.
    fn run_step(
        i: usize,
        alive: Alive,
        prog: RwSignal<Vec<Dir>>,
        puzzle: RwSignal<Puzzle>,
        pos: RwSignal<(i32, i32)>,
        step: RwSignal<Option<usize>>,
        phase: RwSignal<Phase>,
        bonk: RwSignal<bool>,
        msg: RwSignal<String>,
        on_win: Callback<()>,
    ) {
        if !alive.get() {
            return;
        }
        let p = puzzle.get_untracked();
        let program = prog.get_untracked();
        let alive_f = alive.clone();
        let fail = move |text: &str| {
            audio::bump();
            msg.set(text.to_string());
            audio::say(text);
            bonk.set(true);
            let alive = alive_f.clone();
            set_timeout(
                move || {
                    if alive.get() {
                        bonk.set(false);
                        pos.set(puzzle.get_untracked().start);
                        step.set(None);
                        phase.set(Phase::Edit);
                    }
                },
                Duration::from_millis(1100),
            );
        };
        if i >= program.len() {
            if pos.get_untracked() == p.plant {
                on_win.run(());
            } else {
                fail("Presque ! Change le programme.");
            }
            return;
        }
        step.set(Some(i));
        let (dx, dy) = program[i].delta();
        let (x, y) = pos.get_untracked();
        let (nx, ny) = (x + dx, y + dy);
        if nx < 0 || ny < 0 || nx >= p.n || ny >= p.n {
            fail("Oups ! Le robot sort du jardin.");
            return;
        }
        if p.rocks.contains(&(nx, ny)) {
            fail("Bong ! Il y a un caillou.");
            return;
        }
        audio::pop();
        pos.set((nx, ny));
        if (nx, ny) == p.plant {
            // Reached the plant: stop here even if commands remain.
            let alive2 = alive.clone();
            set_timeout(
                move || {
                    if alive2.get() {
                        on_win.run(())
                    }
                },
                Duration::from_millis(400),
            );
            return;
        }
        let alive2 = alive.clone();
        set_timeout(
            move || run_step(i + 1, alive2, prog, puzzle, pos, step, phase, bonk, msg, on_win),
            Duration::from_millis(450),
        );
    }

    let on_win = {
        let alive = alive.clone();
        Callback::new(move |_| {
            audio::good();
            phase.set(Phase::Won);
            step.set(None);
            app.add_stars(2);
            msg.set("Bravo ! La plante a de l'eau 🌸".into());
            audio::say("Bravo ! La plante a de l'eau !");
            let alive = alive.clone();
            set_timeout(
                move || {
                    if !alive.get() {
                        return;
                    }
                    let r = round.get_untracked() + 1;
                    if r >= PUZZLES {
                        finished.set(true);
                    } else {
                        round.set(r);
                        let p = make_puzzle(level, r);
                        pos.set(p.start);
                        puzzle.set(p);
                        prog.set(Vec::new());
                        phase.set(Phase::Edit);
                        msg.set("Nouveau jardin !".into());
                    }
                },
                Duration::from_millis(1600),
            );
        })
    };

    let go = {
        let alive = alive.clone();
        move |_| {
            if phase.get_untracked() != Phase::Edit {
                return;
            }
            if prog.with_untracked(|p| p.is_empty()) {
                msg.set("Ajoute des flèches d'abord !".into());
                audio::say("Ajoute des flèches d'abord !");
                return;
            }
            pos.set(puzzle.get_untracked().start);
            phase.set(Phase::Running);
            msg.set("C'est parti !".into());
            let alive = alive.clone();
            set_timeout(
                move || run_step(0, alive, prog, puzzle, pos, step, phase, bonk, msg, on_win),
                Duration::from_millis(350),
            );
        }
    };

    let again = Callback::new(move |_| {
        round.set(0);
        let p = make_puzzle(level, 0);
        pos.set(p.start);
        puzzle.set(p);
        prog.set(Vec::new());
        phase.set(Phase::Edit);
        finished.set(false);
    });

    let editing = move || phase.get() == Phase::Edit;

    let board = move || {
        let p = puzzle.get();
        let n = p.n;
        let cells = (0..n * n)
            .map(|k| {
                let (x, y) = (k % n, k / n);
                let content = if p.rocks.contains(&(x, y)) { "🪨" } else { "" };
                let class = if (x + y) % 2 == 0 { "gcell a" } else { "gcell b" };
                view! { <div class=class>{content}</div> }
            })
            .collect_view();
        let (px, py) = p.plant;
        view! {
            <div class="garden" style:grid-template-columns=format!("repeat({n}, 1fr)")>
                {cells}
                <div class="sprite" style=format!("left:8px;top:8px;width:calc((100% - 16px) * {f});height:calc((100% - 16px) * {f});transform:translate({}%, {}%);font-size:34px", px * 100, py * 100, f = 1.0 / n as f64)>
                    {move || if phase.get() == Phase::Won { "🌸" } else { "🌱" }}
                </div>
                <div
                    class="sprite robot-sprite"
                    class:bonk=move || bonk.get()
                    style:left="8px"
                    style:top="8px"
                    style:width=format!("calc((100% - 16px) * {})", 1.0 / n as f64)
                    style:height=format!("calc((100% - 16px) * {})", 1.0 / n as f64)
                    style:transform=move || {
                        let (x, y) = pos.get();
                        format!("translate({}%, {}%)", x * 100, y * 100)
                    }
                >
                    "🤖"
                </div>
            </div>
        }
        .into_any()
    };

    view! {
        <div class="screen robot">
            <TopBar>
                <div class="dots">
                    {(0..PUZZLES)
                        .map(|i| view! { <div class="dot" class:done=move || i < round.get() class:cur=move || i == round.get()></div> })
                        .collect_view()}
                </div>
            </TopBar>
            {move || {
                if finished.get() {
                    untrack(|| view! { <Finish title="Super jardinier !" detail="Tous les jardins sont arrosés." stars=5 on_again=again/> }).into_any()
                } else {
                    let go = go.clone();
                    view! {
                        <Cat text=Signal::derive(move || msg.get()) happy=Signal::derive(move || phase.get() == Phase::Won)/>
                        <div class="board-wrap">{board}</div>
                        <div class="program">
                            {(0..SLOTS)
                                .map(|i| {
                                    let cmd = move || prog.with(|p| p.get(i).copied());
                                    view! {
                                        <button
                                            class=move || match cmd() {
                                                Some(d) => format!("slot filled {}", d.class()),
                                                None => "slot".to_string(),
                                            }
                                            class:current=move || step.get() == Some(i)
                                            on:click=move |_| {
                                                if editing() && cmd().is_some() {
                                                    audio::pop();
                                                    prog.update(|p| {
                                                        p.remove(i);
                                                    });
                                                    reset_robot();
                                                }
                                            }
                                        >
                                            {move || cmd().map(|d| d.arrow()).unwrap_or("")}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                        <div class="cmds">
                            <button class="cmd c-go" disabled=move || !editing() on:click=go>"Go ▶"</button>
                            {[Dir::Left, Dir::Up, Dir::Down, Dir::Right]
                                .into_iter()
                                .map(|d| view! { <button class=format!("cmd {}", d.class()) disabled=move || !editing() on:click=move |_| add(d)>{d.arrow()}</button> })
                                .collect_view()}
                        </div>
                        <div class="robot-tools">
                            <button class="big-btn orange small" disabled=move || !editing() on:click=move |_| { audio::pop(); prog.set(Vec::new()); reset_robot(); }>"🧽 Tout effacer"</button>
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
