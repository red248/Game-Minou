use crate::{app, audio, Game, Screen};
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    let app = app();
    let hello = move || {
        let name = app.settings.with(|s| s.name.trim().to_string());
        if name.is_empty() {
            "Apprends en t'amusant !".to_string()
        } else {
            format!("Bonjour {name} !")
        }
    };
    let games = move || {
        let hidden = app.settings.with(|s| s.hidden.clone());
        Game::ALL
            .into_iter()
            .filter(|g| !hidden.iter().any(|h| h == g.id()))
            .map(|g| {
                view! {
                    <button
                        class="card"
                        style:background=format!("linear-gradient(160deg, {} 0%, #ffffff 160%)", g.color())
                        on:click=move |_| {
                            audio::pop();
                            app.open(g)
                        }
                    >
                        <span class="card-emoji">{g.emoji()}</span>
                        <span class="card-name">{g.name()}</span>
                    </button>
                }
            })
            .collect_view()
    };
    view! {
        <div class="screen home">
            <header class="home-head">
                <div class="logo">
                    <span class="logo-cat">"🐱"</span>
                    <div>
                        <div class="logo-name">"Minou"</div>
                        <div class="logo-sub">{hello}</div>
                    </div>
                </div>
                <button class="parents-btn" on:click=move |_| app.screen.set(Screen::Parents)>
                    "⚙️"<small>"Parents"</small>
                </button>
            </header>
            <div class="home-stars">
                <div class="stars-pill big">"⭐ " {move || app.stats.with(|s| s.stars)} " étoiles"</div>
            </div>
            <div class="cards">{games}</div>
        </div>
    }
}
