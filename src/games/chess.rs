use crate::ui::TopBar;
use crate::util::{rand_f, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::time::Duration;

/// 64 squares, index = row * 8 + col, row 0 = rank 8 (robot side).
/// Positive = player (white), negative = robot (black).
/// 1 pawn, 2 knight, 3 bishop, 4 rook, 5 queen, 6 king.
type Board = [i8; 64];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Mv {
    from: usize,
    to: usize,
}

fn initial() -> Board {
    let back = [4, 2, 3, 5, 6, 3, 2, 4];
    let mut b = [0i8; 64];
    for c in 0..8 {
        b[c] = -back[c];
        b[8 + c] = -1;
        b[48 + c] = 1;
        b[56 + c] = back[c];
    }
    b
}

fn on_board(r: i32, c: i32) -> bool {
    (0..8).contains(&r) && (0..8).contains(&c)
}

fn pseudo(b: &Board, white: bool) -> Vec<Mv> {
    let mut out = Vec::with_capacity(48);
    let mine = |p: i8| p != 0 && (p > 0) == white;
    let theirs = |p: i8| p != 0 && (p > 0) != white;
    for from in 0..64 {
        let p = b[from];
        if !mine(p) {
            continue;
        }
        let (r, c) = ((from / 8) as i32, (from % 8) as i32);
        let at = |r: i32, c: i32| (r * 8 + c) as usize;
        let slide = |dirs: &[(i32, i32)], out: &mut Vec<Mv>| {
            for &(dr, dc) in dirs {
                let (mut nr, mut nc) = (r + dr, c + dc);
                while on_board(nr, nc) {
                    let t = b[at(nr, nc)];
                    if mine(t) {
                        break;
                    }
                    out.push(Mv { from, to: at(nr, nc) });
                    if theirs(t) {
                        break;
                    }
                    nr += dr;
                    nc += dc;
                }
            }
        };
        const DIAG: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        const ORTH: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        match p.abs() {
            1 => {
                let dir = if white { -1 } else { 1 };
                let start = if white { 6 } else { 1 };
                if on_board(r + dir, c) && b[at(r + dir, c)] == 0 {
                    out.push(Mv { from, to: at(r + dir, c) });
                    if r == start && b[at(r + 2 * dir, c)] == 0 {
                        out.push(Mv { from, to: at(r + 2 * dir, c) });
                    }
                }
                for dc in [-1, 1] {
                    if on_board(r + dir, c + dc) && theirs(b[at(r + dir, c + dc)]) {
                        out.push(Mv { from, to: at(r + dir, c + dc) });
                    }
                }
            }
            2 => {
                for (dr, dc) in [(1, 2), (2, 1), (-1, 2), (-2, 1), (1, -2), (2, -1), (-1, -2), (-2, -1)] {
                    if on_board(r + dr, c + dc) && !mine(b[at(r + dr, c + dc)]) {
                        out.push(Mv { from, to: at(r + dr, c + dc) });
                    }
                }
            }
            3 => slide(&DIAG, &mut out),
            4 => slide(&ORTH, &mut out),
            5 => {
                slide(&DIAG, &mut out);
                slide(&ORTH, &mut out);
            }
            _ => {
                for (dr, dc) in DIAG.iter().chain(ORTH.iter()) {
                    if on_board(r + dr, c + dc) && !mine(b[at(r + dr, c + dc)]) {
                        out.push(Mv { from, to: at(r + dr, c + dc) });
                    }
                }
            }
        }
    }
    out
}

fn make(b: &Board, m: Mv) -> Board {
    let mut nb = *b;
    let p = nb[m.from];
    nb[m.to] = p;
    nb[m.from] = 0;
    let row = m.to / 8;
    if p.abs() == 1 && (row == 0 || row == 7) {
        nb[m.to] = 5 * p.signum();
    }
    nb
}

fn attacked(b: &Board, sq: usize, by_white: bool) -> bool {
    pseudo(b, by_white).iter().any(|m| m.to == sq)
}

fn king(b: &Board, white: bool) -> Option<usize> {
    b.iter().position(|&p| p == if white { 6 } else { -6 })
}

fn in_check(b: &Board, white: bool) -> bool {
    king(b, white).is_some_and(|k| attacked(b, k, !white))
}

fn legal(b: &Board, white: bool) -> Vec<Mv> {
    pseudo(b, white).into_iter().filter(|m| !in_check(&make(b, *m), white)).collect()
}

fn value(p: i8) -> f64 {
    [0.0, 1.0, 3.0, 3.0, 5.0, 9.0, 50.0][p.unsigned_abs() as usize]
}

fn score(b: &Board, m: Mv, white: bool) -> f64 {
    let nb = make(b, m);
    let mut s = value(b[m.to]) * 10.0;
    if b[m.from].abs() == 1 && nb[m.to].abs() == 5 {
        s += 80.0;
    }
    let opp = !white;
    let replies = legal(&nb, opp);
    let check = in_check(&nb, opp);
    if replies.is_empty() {
        s += if check { 10_000.0 } else { -60.0 };
    }
    if attacked(&nb, m.to, opp) {
        s -= value(nb[m.to]) * 9.0;
    }
    if check {
        s += 2.0;
    }
    // Small preference for central squares.
    let (r, c) = ((m.to / 8) as f64, (m.to % 8) as f64);
    s += 1.0 - ((r - 3.5).abs() + (c - 3.5).abs()) * 0.15;
    s
}

fn best(b: &Board, white: bool, noise: f64) -> Option<Mv> {
    legal(b, white)
        .into_iter()
        .map(|m| (m, score(b, m, white) + rand_f() * noise))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|x| x.0)
}

fn glyph(p: i8) -> String {
    let g = ["", "♟", "♞", "♝", "♜", "♛", "♚"][p.unsigned_abs() as usize];
    format!("{g}\u{FE0E}")
}

fn captured(b: &Board, white: bool) -> String {
    // Pieces of the given colour missing from the board.
    let start = [0, 8, 2, 2, 2, 1, 1];
    let mut s = String::new();
    for kind in [5i8, 4, 3, 2, 1] {
        let n = b.iter().filter(|&&p| p == if white { kind } else { -kind }).count();
        for _ in n..start[kind as usize] {
            s.push_str(&glyph(kind));
        }
    }
    s
}

#[derive(Clone, Copy, PartialEq)]
enum Over {
    No,
    Win,
    Lose,
    Draw,
}

#[component]
pub fn Chess() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let noise = match app.level() {
        1 | 2 => 16.0,
        3 => 8.0,
        _ => 3.0,
    };
    let board = RwSignal::new(initial());
    let sel = RwSignal::new(None::<usize>);
    let last = RwSignal::new(None::<Mv>);
    let hint = RwSignal::new(None::<Mv>);
    let my_turn = RwSignal::new(true);
    let over = RwSignal::new(Over::No);
    let status = RwSignal::new("À toi !".to_string());
    let history = RwSignal::new(Vec::<(Board, Option<Mv>)>::new());

    let targets = Memo::new(move |_| match sel.get() {
        Some(s) => legal(&board.get(), true).into_iter().filter(|m| m.from == s).map(|m| m.to).collect(),
        None => Vec::<usize>::new(),
    });

    // After a move, look at the position from the side to move.
    let check_end = move |b: &Board, white_to_move: bool| -> bool {
        let moves = legal(b, white_to_move);
        let chk = in_check(b, white_to_move);
        if moves.is_empty() {
            if chk && !white_to_move {
                over.set(Over::Win);
                status.set("Échec et mat ! Tu as gagné ! 🎉".into());
                audio::win();
                audio::say("Échec et mat ! Bravo, tu as gagné !");
                app.add_stars(10);
            } else if chk {
                over.set(Over::Lose);
                status.set("Échec et mat… Le robot gagne cette fois !".into());
                audio::say("Échec et mat. Le robot a gagné cette fois. On rejoue ?");
            } else {
                over.set(Over::Draw);
                status.set("Pat ! Match nul 🤝".into());
                audio::say("Pat ! Match nul.");
            }
            return true;
        }
        let only_kings = b.iter().all(|&p| p == 0 || p.abs() == 6);
        if only_kings {
            over.set(Over::Draw);
            status.set("Il ne reste que les rois : match nul 🤝".into());
            return true;
        }
        if chk {
            status.set(if white_to_move { "Échec ! Protège ton roi !".into() } else { "Échec au robot !".into() });
            audio::say("Échec !");
        }
        false
    };

    let robot_play = {
        let alive = alive.clone();
        move || {
            status.set("Je réfléchis…".into());
            let alive = alive.clone();
            set_timeout(
                move || {
                    if !alive.get() {
                        return;
                    }
                    let b = board.get_untracked();
                    if let Some(m) = best(&b, false, noise) {
                        let nb = make(&b, m);
                        if b[m.to] != 0 {
                            audio::bump();
                        } else {
                            audio::pop();
                        }
                        board.set(nb);
                        last.set(Some(m));
                        my_turn.set(true);
                        status.set("À toi !".into());
                        check_end(&nb, true);
                    }
                },
                Duration::from_millis(700),
            );
        }
    };

    let tap = move |sq: usize| {
        if !my_turn.get_untracked() || over.get_untracked() != Over::No {
            return;
        }
        let b = board.get_untracked();
        if b[sq] > 0 {
            audio::pop();
            sel.set(if sel.get_untracked() == Some(sq) { None } else { Some(sq) });
            return;
        }
        let Some(from) = sel.get_untracked() else { return };
        let m = Mv { from, to: sq };
        if !legal(&b, true).contains(&m) {
            sel.set(None);
            return;
        }
        history.update(|h| h.push((b, last.get_untracked())));
        let nb = make(&b, m);
        if b[sq] != 0 {
            audio::good();
            app.add_stars(1);
        } else {
            audio::pop();
        }
        board.set(nb);
        last.set(Some(m));
        sel.set(None);
        hint.set(None);
        if !check_end(&nb, false) {
            my_turn.set(false);
            robot_play();
        }
    };

    let undo = move |_| {
        if !my_turn.get_untracked() && over.get_untracked() == Over::No {
            return;
        }
        if let Some((b, l)) = history.try_update(|h| h.pop()).flatten() {
            board.set(b);
            last.set(l);
            sel.set(None);
            hint.set(None);
            over.set(Over::No);
            my_turn.set(true);
            status.set("On recommence ce coup !".into());
        }
    };

    let show_hint = move |_| {
        if !my_turn.get_untracked() || over.get_untracked() != Over::No {
            return;
        }
        let m = best(&board.get_untracked(), true, 0.0);
        hint.set(m);
        sel.set(m.map(|m| m.from));
        status.set("Regarde la case bleue 💡".into());
    };

    let new_game = move |_| {
        board.set(initial());
        sel.set(None);
        last.set(None);
        hint.set(None);
        history.set(Vec::new());
        over.set(Over::No);
        my_turn.set(true);
        status.set("À toi !".into());
    };

    let squares = (0..64)
        .map(|sq| {
            let tap = tap.clone();
            let light = (sq / 8 + sq % 8) % 2 == 0;
            view! {
                <button
                    class="sq"
                    class:l=light
                    class:d=!light
                    class:sel=move || sel.get() == Some(sq)
                    class:last=move || last.get().is_some_and(|m| m.from == sq || m.to == sq)
                    class:hint=move || hint.get().is_some_and(|m| m.from == sq || m.to == sq)
                    class:check=move || {
                        let b = board.get();
                        b[sq].abs() == 6 && in_check(&b, b[sq] > 0)
                    }
                    on:click=move |_| tap(sq)
                >
                    {move || {
                        let p = board.with(|b| b[sq]);
                        (p != 0).then(|| view! { <span class="pc" class:w={p > 0} class:b={p < 0}>{glyph(p)}</span> })
                    }}
                    {move || {
                        let t = targets.with(|t| t.contains(&sq));
                        let occupied = board.with(|b| b[sq] != 0);
                        match (t, occupied) {
                            (true, false) => Some(view! { <span class="dotm"></span> }.into_any()),
                            (true, true) => Some(view! { <span class="ring"></span> }.into_any()),
                            _ => None,
                        }
                    }}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="screen chess">
            <TopBar/>
            <div class="chess-head">
                <div class="chess-robot" class:think=move || !my_turn.get()>"🤖"</div>
                <div class="bubble">{move || status.get()}</div>
            </div>
            <div class="captured">{move || board.with(|b| captured(b, true))}</div>
            <div class="board">{squares}</div>
            <div class="captured">{move || board.with(|b| captured(b, false))}</div>
            <div class="chess-bar">
                <button class="round" style="background:#ffc93c" on:click=show_hint aria-label="Indice">"💡"</button>
                {move || {
                    if over.get() != Over::No {
                        view! { <button class="big-btn green small" on:click=new_game>"🔄 Nouvelle partie"</button> }.into_any()
                    } else {
                        view! { <div class="chess-status">{move || if my_turn.get() { "À toi de jouer !" } else { "Le robot joue…" }}</div> }.into_any()
                    }
                }}
                <button class="round" style="background:#5cb8ff;color:white" on:click=undo aria-label="Annuler">"↩"</button>
            </div>
        </div>
    }
}
