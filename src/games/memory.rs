use crate::ui::{Finish, TopBar};
use crate::util::{shuffle, Alive};
use crate::{app, audio};
use leptos::prelude::*;
use std::time::Duration;

const ANIMALS: &[&str] = &["🐻", "🐶", "🐰", "🐸", "🐭", "🐷", "🦊", "🐼", "🐨", "🦁", "🐯", "🐮", "🐵", "🐔", "🐙", "🦄"];

#[derive(Clone, Copy, PartialEq)]
struct Card {
    face: &'static str,
    up: bool,
    found: bool,
}

fn deal(pairs: usize) -> Vec<Card> {
    let mut faces: Vec<&'static str> = ANIMALS.to_vec();
    shuffle(&mut faces);
    let mut cards: Vec<Card> = faces[..pairs]
        .iter()
        .flat_map(|f| [Card { face: f, up: false, found: false }; 2])
        .collect();
    shuffle(&mut cards);
    cards
}

#[component]
pub fn Memory() -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let (pairs, cols) = match app.level() {
        1 => (3, 3),
        2 => (6, 4),
        3 => (6, 4),
        4 => (8, 4),
        _ => (10, 4),
    };
    let cards = RwSignal::new(deal(pairs));
    let open = RwSignal::new(Vec::<usize>::new());
    let moves = RwSignal::new(0u32);
    let done = RwSignal::new(false);
    let lock = RwSignal::new(false);

    let flip = move |i: usize| {
        if lock.get_untracked() || cards.with_untracked(|c| c[i].up || c[i].found) {
            return;
        }
        audio::pop();
        cards.update(|c| c[i].up = true);
        open.update(|o| o.push(i));
        let o = open.get_untracked();
        if o.len() < 2 {
            return;
        }
        moves.update(|m| *m += 1);
        let (a, b) = (o[0], o[1]);
        open.set(Vec::new());
        if cards.with_untracked(|c| c[a].face == c[b].face) {
            audio::good();
            app.add_stars(1);
            cards.update(|c| {
                c[a].found = true;
                c[b].found = true;
            });
            if cards.with_untracked(|c| c.iter().all(|c| c.found)) {
                let alive = alive.clone();
                set_timeout(
                    move || {
                        if alive.get() {
                            done.set(true)
                        }
                    },
                    Duration::from_millis(700),
                );
            }
        } else {
            lock.set(true);
            let alive = alive.clone();
            set_timeout(
                move || {
                    if alive.get() {
                        cards.update(|c| {
                            c[a].up = false;
                            c[b].up = false;
                        });
                        lock.set(false);
                    }
                },
                Duration::from_millis(900),
            );
        }
    };

    let again = Callback::new(move |_| {
        cards.set(deal(pairs));
        moves.set(0);
        done.set(false);
    });

    view! {
        <div class="screen">
            <TopBar/>
            <div class="moves">"Coups : " {move || moves.get()}</div>
            {move || {
                if done.get() {
                    let m = moves.get_untracked();
                    let stars = if m as usize <= pairs + 2 { 5 } else if m as usize <= pairs * 2 { 3 } else { 1 };
                    untrack(|| {
                        view! { <Finish title="Bravo, tout est trouvé !" detail=format!("En {m} coups") stars=stars on_again=again/> }
                    })
                    .into_any()
                } else {
                    let flip = flip.clone();
                    view! {
                        <div class="memory-grid" style:grid-template-columns=format!("repeat({cols}, 1fr)")>
                            {(0..pairs * 2)
                                .map(|i| {
                                    let flip = flip.clone();
                                    view! {
                                        <button
                                            class="mcard"
                                            class:up=move || cards.with(|c| c[i].up || c[i].found)
                                            class:found=move || cards.with(|c| c[i].found)
                                            on:click=move |_| flip(i)
                                        >
                                            <div class="mcard-inner">
                                                <div class="mface mback">"?"</div>
                                                <div class="mface mfront">{move || cards.with(|c| c[i].face)}</div>
                                            </div>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
