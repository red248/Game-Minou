use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

mod audio;
mod home;
mod parents;
mod questions;
mod quiz;
mod store;
mod ui;
mod util;
mod games {
    pub mod asteroids;
    pub mod chess;
    pub mod dino;
    pub mod maze;
    pub mod memory;
    pub mod robot;
}

use store::{Settings, Stats};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Game {
    Reading,
    Letters,
    Math,
    Memory,
    Robot,
    Maze,
    Asteroids,
    Dino,
    Chess,
}

impl Game {
    pub const ALL: [Game; 9] = [
        Game::Reading,
        Game::Math,
        Game::Asteroids,
        Game::Dino,
        Game::Letters,
        Game::Memory,
        Game::Robot,
        Game::Maze,
        Game::Chess,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Game::Reading => "lecture",
            Game::Letters => "lettres",
            Game::Math => "calculs",
            Game::Memory => "paires",
            Game::Robot => "robot",
            Game::Maze => "labyrinthe",
            Game::Asteroids => "asteroides",
            Game::Dino => "dino",
            Game::Chess => "echecs",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Game::Reading => "Lecture",
            Game::Letters => "Les lettres",
            Game::Math => "Calculs",
            Game::Memory => "Les paires",
            Game::Robot => "Robot jardinier",
            Game::Maze => "Labyrinthes",
            Game::Asteroids => "Astéroïdes",
            Game::Dino => "Le Dino",
            Game::Chess => "Échecs",
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Game::Reading => "📖",
            Game::Letters => "🔤",
            Game::Math => "🍎",
            Game::Memory => "🃏",
            Game::Robot => "🤖",
            Game::Maze => "🏴‍☠️",
            Game::Asteroids => "🚀",
            Game::Dino => "🦕",
            Game::Chess => "♟️",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Game::Reading => "#ffd6a5",
            Game::Letters => "#cdb4ff",
            Game::Math => "#a0e7d9",
            Game::Memory => "#ffc6e0",
            Game::Robot => "#b8f2a6",
            Game::Maze => "#ffe48a",
            Game::Asteroids => "#ff9e9e",
            Game::Dino => "#9fd8ff",
            Game::Chess => "#d9d2ff",
        }
    }

    /// Action games are preceded by the "wall of questions".
    pub fn is_action(self) -> bool {
        matches!(self, Game::Asteroids | Game::Dino)
    }

    pub fn from_id(id: &str) -> Option<Game> {
        Game::ALL.into_iter().find(|g| g.id() == id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    Parents,
    Gate(Game),
    Play(Game),
    Limit,
}

#[derive(Clone, Copy)]
pub struct App {
    pub screen: RwSignal<Screen>,
    pub settings: RwSignal<Settings>,
    pub stats: RwSignal<Stats>,
}

pub fn app() -> App {
    expect_context::<App>()
}

impl App {
    pub fn limit_reached(&self) -> bool {
        let limit = self.settings.with_untracked(|s| s.daily_limit_min);
        limit > 0 && self.stats.with_untracked(|s| s.today_seconds()) >= limit * 60
    }

    pub fn open(&self, g: Game) {
        audio::unlock();
        if self.limit_reached() {
            self.screen.set(Screen::Limit);
            return;
        }
        self.stats.update(|s| *s.plays.entry(g.id().to_string()).or_default() += 1);
        let gate = self.settings.with_untracked(|s| s.gate_questions);
        if g.is_action() && gate > 0 {
            self.screen.set(Screen::Gate(g));
        } else {
            self.screen.set(Screen::Play(g));
        }
    }

    pub fn home(&self) {
        audio::stop_speech();
        let _ = self.screen.try_set(Screen::Home);
    }

    pub fn add_stars(&self, n: u32) {
        let _ = self.stats.try_update(|s| s.stars += n);
    }

    pub fn record(&self, domain: &str, ok: bool) {
        let _ = self.stats.try_update(|s| {
            let map = if ok { &mut s.ok } else { &mut s.ko };
            *map.entry(domain.to_string()).or_default() += 1;
        });
    }

    pub fn level(&self) -> u8 {
        self.settings.with_untracked(|s| s.level)
    }
}

#[component]
fn Root() -> impl IntoView {
    let app = App {
        screen: RwSignal::new(Screen::Home),
        settings: RwSignal::new(store::load_settings()),
        stats: RwSignal::new(store::load_stats()),
    };
    provide_context(app);

    Effect::new(move |_| app.settings.with(store::save_settings));
    Effect::new(move |_| app.stats.with(store::save_stats));
    Effect::new(move |_| audio::set_enabled(app.settings.with(|s| s.sound)));

    // Count play time (only while a game is open and the app is visible).
    set_interval(
        move || {
            let screen = app.screen.get_untracked();
            if !matches!(screen, Screen::Gate(_) | Screen::Play(_)) || !util::visible() {
                return;
            }
            let today = util::today();
            app.stats.update(|s| *s.days.entry(today).or_default() += 1);
            if app.limit_reached() {
                audio::say("C'est fini pour aujourd'hui. À demain !");
                app.screen.set(Screen::Limit);
            }
        },
        Duration::from_secs(1),
    );

    move || {
        let screen = app.screen.get();
        untrack(move || match screen {
            Screen::Home => view! { <home::Home/> }.into_any(),
            Screen::Parents => view! { <parents::Parents/> }.into_any(),
            Screen::Limit => view! { <ui::LimitScreen/> }.into_any(),
            Screen::Gate(g) => view! { <quiz::Gate game=g/> }.into_any(),
            Screen::Play(g) => match g {
                Game::Reading | Game::Letters | Game::Math => view! { <quiz::Learn game=g/> }.into_any(),
                Game::Memory => view! { <games::memory::Memory/> }.into_any(),
                Game::Robot => view! { <games::robot::Robot/> }.into_any(),
                Game::Maze => view! { <games::maze::Maze/> }.into_any(),
                Game::Asteroids => view! { <games::asteroids::Asteroids/> }.into_any(),
                Game::Dino => view! { <games::dino::Dino/> }.into_any(),
                Game::Chess => view! { <games::chess::Chess/> }.into_any(),
            },
        })
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    if let Some(el) = util::document().get_element_by_id("loading") {
        el.remove();
    }
    mount_to_body(Root);
}
