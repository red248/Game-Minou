use crate::{app, audio, Screen};
use leptos::prelude::*;

/// Top bar with a back button, the star counter and optional extra content.
#[component]
pub fn TopBar(#[prop(optional)] children: Option<Children>, #[prop(optional)] dark: bool) -> impl IntoView {
    let app = app();
    view! {
        <div class="topbar" class:dark=dark>
            <button class="round back" on:click=move |_| app.home() aria-label="Retour">"←"</button>
            <div class="stars-pill">"⭐ " {move || app.stats.with(|s| s.stars)}</div>
            <div class="topbar-extra">{children.map(|c| c())}</div>
        </div>
    }
}

/// The cat mascot with a speech bubble.
#[component]
pub fn Cat(#[prop(into)] text: Signal<String>, #[prop(optional)] happy: Option<Signal<bool>>) -> impl IntoView {
    let happy = happy.unwrap_or_else(|| Signal::stored(false));
    view! {
        <div class="cat-row">
            <div class="cat" class:jump=move || happy.get()>{move || if happy.get() { "😻" } else { "🐱" }}</div>
            <div class="bubble">{move || text.get()}</div>
        </div>
    }
}

/// End of a round.
#[component]
pub fn Finish(
    #[prop(into)] title: String,
    #[prop(into)] detail: String,
    stars: u32,
    on_again: Callback<()>,
) -> impl IntoView {
    let app = app();
    audio::win();
    audio::say(&title);
    let star_line: String = "⭐".repeat(stars.clamp(1, 5) as usize);
    view! {
        <div class="finish pop-in">
            <div class="finish-cat">"😻"</div>
            <h1>{title}</h1>
            <div class="finish-stars">{star_line}</div>
            <p>{detail}</p>
            <div class="finish-buttons">
                <button class="big-btn green" on:click=move |_| on_again.run(())>"🔄 Rejouer"</button>
                <button class="big-btn blue" on:click=move |_| app.home()>"🏠 Menu"</button>
            </div>
        </div>
    }
}

#[component]
pub fn LimitScreen() -> impl IntoView {
    let app = app();
    view! {
        <div class="screen limit">
            <div class="limit-moon">"🌙"</div>
            <div class="limit-cat">"😴"</div>
            <h1>"C'est fini pour aujourd'hui !"</h1>
            <p>"Minou est fatigué. On rejoue demain ?"</p>
            <button class="big-btn blue" on:click=move |_| app.screen.set(Screen::Home)>"🏠 Menu"</button>
        </div>
    }
}

/// Renders a choice label: "A|B" = picture A with badge B.
pub fn choice_label(s: &str) -> AnyView {
    if let Some((pic, badge)) = s.split_once('|') {
        let many = pic.chars().filter(|c| *c as u32 >= 0x1F000).count() > 1;
        view! {
            <span class="pic">
                <span class="pic-main" class:two=many>{pic.to_string()}</span>
                <span class="pic-badge">{badge.to_string()}</span>
            </span>
        }
        .into_any()
    } else {
        view! { <span class="label">{s.to_string()}</span> }.into_any()
    }
}

/// Hearts + progress bar for the action games.
#[component]
pub fn Hud(hearts: RwSignal<u32>, max: u32, progress: RwSignal<f64>, #[prop(into)] label: String) -> impl IntoView {
    view! {
        <div class="hud">
            <div class="hud-label">{label}</div>
            <div class="hud-bar"><div class="hud-fill" style:width=move || format!("{:.1}%", progress.get() * 100.0)></div></div>
            <div class="hearts">
                {move || {
                    let h = hearts.get();
                    (0..max).map(|i| if i < h { "❤️" } else { "🤍" }).collect::<String>()
                }}
            </div>
        </div>
    }
}
