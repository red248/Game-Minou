use crate::store::Stats;
use crate::util::{day_offset, range};
use crate::{app, audio, Game};
use leptos::prelude::*;

const LEVELS: [(u8, &str); 5] = [
    (1, "4 ans · compter jusqu'à 5, mots simples, lettres"),
    (2, "5 ans · compter jusqu'à 10, premières phrases"),
    (3, "6 ans · additions jusqu'à 10, phrases"),
    (4, "6-7 ans · additions et soustractions jusqu'à 10"),
    (5, "7 ans · calculs jusqu'à 20, phrases longues"),
];

#[component]
pub fn Parents() -> impl IntoView {
    let app = app();
    let unlocked = RwSignal::new(false);
    let a = range(6, 9);
    let b = range(3, 9);
    let input = RwSignal::new(String::new());
    let shake = RwSignal::new(false);

    let press = move |k: &'static str| {
        audio::pop();
        match k {
            "⌫" => input.update(|s| {
                s.pop();
            }),
            "OK" => {
                if input.get_untracked().parse::<i32>().ok() == Some(a * b) {
                    unlocked.set(true);
                } else {
                    audio::bad();
                    input.set(String::new());
                    shake.set(true);
                    set_timeout(move || { let _ = shake.try_set(false); }, std::time::Duration::from_millis(500));
                }
            }
            d => input.update(|s| {
                if s.len() < 3 {
                    s.push_str(d)
                }
            }),
        }
    };

    let lock = move || {
        let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "⌫", "0", "OK"];
        view! {
            <div class="adult-gate">
                <p>"Pour les grands : combien font"</p>
                <div class="gate-q">{format!("{a} × {b} ?")}</div>
                <div class="gate-input" class:shake=move || shake.get()>{move || input.get()}" "</div>
                <div class="keypad">
                    {keys.into_iter().map(|k| view! { <button on:click=move |_| press(k)>{k}</button> }).collect_view()}
                </div>
            </div>
        }
    };

    view! {
        <div class="screen parents">
            <div class="parents-top">
                <button class="round back" on:click=move |_| app.home()>"←"</button>
                <h2>"⚙️ Espace parents"</h2>
            </div>
            <div class="parents-body">
                {move || if unlocked.get() { view! { <Panel/> }.into_any() } else { lock().into_any() }}
            </div>
        </div>
    }
}

fn pct(s: &Stats, d: &str) -> String {
    let ok = s.ok.get(d).copied().unwrap_or(0);
    let ko = s.ko.get(d).copied().unwrap_or(0);
    if ok + ko == 0 {
        "—".into()
    } else {
        format!("{} % ({} questions)", ok * 100 / (ok + ko), ok + ko)
    }
}

fn mins(sec: u32) -> String {
    if sec < 60 {
        format!("{sec} s")
    } else {
        format!("{} min", sec / 60)
    }
}

#[component]
fn Panel() -> impl IntoView {
    let app = app();
    let st = app.settings;
    let confirm_reset = RwSignal::new(false);

    let seg = move |opts: Vec<(u32, String)>, get: fn(&crate::store::Settings) -> u32, set: fn(&mut crate::store::Settings, u32)| {
        view! {
            <div class="seg">
                {opts
                    .into_iter()
                    .map(|(v, label)| {
                        view! {
                            <button class:on=move || st.with(|s| get(s) == v) on:click=move |_| st.update(|s| set(s, v))>
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        }
    };

    let week = move || {
        let s = app.stats.get();
        let days: Vec<(String, u32)> = (0..7u32)
            .rev()
            .map(|i| {
                let d = day_offset(i);
                let v = s.days.get(&d).copied().unwrap_or(0);
                (d[8..].to_string(), v)
            })
            .collect();
        let max = days.iter().map(|d| d.1).max().unwrap_or(0).max(60);
        days.into_iter()
            .map(|(d, v)| {
                let h = (v as f64 / max as f64 * 100.0).max(2.0);
                view! {
                    <div class="bar-col">
                        <div class="bar-val">{if v >= 60 { format!("{}", v / 60) } else { String::new() }}</div>
                        <div class="bar" style:height=format!("{h}%")></div>
                        <div class="bar-day">{d}</div>
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <section>
            <h3>"👦 L'enfant"</h3>
            <label class="field">
                "Prénom"
                <input
                    type="text"
                    placeholder="Prénom de l'enfant"
                    prop:value=move || st.with(|s| s.name.clone())
                    on:input=move |ev| st.update(|s| s.name = event_target_value(&ev))
                />
            </label>
        </section>

        <section>
            <h3>"🎓 Niveau"</h3>
            <div class="levels">
                {LEVELS
                    .into_iter()
                    .map(|(l, txt)| {
                        view! {
                            <button class="level" class:on=move || st.with(|s| s.level == l) on:click=move |_| st.update(|s| s.level = l)>
                                <b>{format!("Niveau {l}")}</b>
                                <span>{txt}</span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>

        <section>
            <h3>"🧱 Questions avant un jeu d'action"</h3>
            <p class="hint">"Avant les Astéroïdes et le Dino, l'enfant répond à quelques questions à son niveau."</p>
            {seg(
                [0, 2, 3, 5, 8, 12].iter().map(|&n| (n, if n == 0 { "Aucune".to_string() } else { n.to_string() })).collect(),
                |s| s.gate_questions as u32,
                |s, v| s.gate_questions = v as u8,
            )}
        </section>

        <section>
            <h3>"⏰ Temps de jeu par jour"</h3>
            {seg(
                [0, 10, 15, 20, 30, 45, 60].iter().map(|&n| (n, if n == 0 { "Sans limite".to_string() } else { format!("{n} min") })).collect(),
                |s| s.daily_limit_min,
                |s, v| s.daily_limit_min = v,
            )}
        </section>

        <section>
            <h3>"🔊 Son et voix"</h3>
            {seg(
                vec![(1, "Activés".to_string()), (0, "Coupés".to_string())],
                |s| s.sound as u32,
                |s, v| s.sound = v == 1,
            )}
        </section>

        <section>
            <h3>"🎮 Jeux affichés"</h3>
            <div class="toggles">
                {Game::ALL
                    .into_iter()
                    .map(|g| {
                        let shown = move || st.with(|s| !s.hidden.iter().any(|h| h == g.id()));
                        view! {
                            <button
                                class="toggle"
                                class:on=shown
                                on:click=move |_| {
                                    st.update(|s| {
                                        if let Some(p) = s.hidden.iter().position(|h| h == g.id()) {
                                            s.hidden.remove(p);
                                        } else {
                                            s.hidden.push(g.id().to_string());
                                        }
                                    })
                                }
                            >
                                <span>{g.emoji()}" "{g.name()}</span>
                                <span class="switch"></span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>

        <section>
            <h3>"📊 Statistiques"</h3>
            <div class="stat-grid">
                <div class="stat"><b>{move || mins(app.stats.with(|s| s.today_seconds()))}</b><span>"aujourd'hui"</span></div>
                <div class="stat"><b>{move || app.stats.with(|s| s.days.values().filter(|v| **v >= 30).count())}</b><span>"jours joués"</span></div>
                <div class="stat"><b>{move || app.stats.with(|s| s.stars)}</b><span>"étoiles"</span></div>
            </div>
            <p class="hint">"Minutes de jeu sur 7 jours"</p>
            <div class="week">{week}</div>
            <table class="results">
                <tr><td>"📖 Lecture"</td><td>{move || app.stats.with(|s| pct(s, "lecture"))}</td></tr>
                <tr><td>"🔤 Lettres"</td><td>{move || app.stats.with(|s| pct(s, "lettres"))}</td></tr>
                <tr><td>"🍎 Calculs"</td><td>{move || app.stats.with(|s| pct(s, "calculs"))}</td></tr>
                <tr><td>"⚖️ Logique"</td><td>{move || app.stats.with(|s| pct(s, "logique"))}</td></tr>
            </table>
            <p class="hint">"Bonnes réponses du premier coup."</p>
            <table class="results">
                {Game::ALL
                    .into_iter()
                    .map(|g| {
                        view! {
                            <tr>
                                <td>{g.emoji()}" "{g.name()}</td>
                                <td>{move || {
                                    let n = app.stats.with(|s| s.plays.get(g.id()).copied().unwrap_or(0));
                                    format!("{n} partie{}", if n > 1 { "s" } else { "" })
                                }}</td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </table>
        </section>

        <section>
            <h3>"♻️ Remise à zéro"</h3>
            <button
                class="danger"
                on:click=move |_| {
                    if confirm_reset.get_untracked() {
                        app.stats.set(Stats::default());
                        confirm_reset.set(false);
                    } else {
                        confirm_reset.set(true);
                    }
                }
            >
                {move || if confirm_reset.get() { "Appuyez encore pour confirmer" } else { "Effacer les étoiles et les statistiques" }}
            </button>
        </section>
        <p class="hint center">"Minou · les données restent sur ce téléphone."</p>
    }
}
