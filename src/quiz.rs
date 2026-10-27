use crate::questions::{self, Prompt, Q};
use crate::ui::{choice_label, Cat, Finish, TopBar};
use crate::util::{pick, Alive};
use crate::{app, audio, Game, Screen};
use leptos::prelude::*;
use std::time::Duration;

fn render_prompt(p: Prompt, say: String) -> AnyView {
    match p {
        Prompt::None => view! {
            <button class="listen" on:click=move |_| audio::say(&say)>"🔊"<small>"Écoute"</small></button>
        }
        .into_any(),
        Prompt::Read(text) => {
            let words: Vec<String> = text.split(' ').map(|w| w.to_string()).collect();
            let full = text.clone();
            view! {
                <div class="reading">
                    <div class="sentence">
                        {words
                            .into_iter()
                            .map(|w| {
                                let spoken = w.trim_matches(|c: char| c == '.' || c == ',').to_string();
                                view! { <button class="word" on:click=move |_| audio::say(&spoken)>{w}</button> }
                            })
                            .collect_view()}
                    </div>
                    <button class="help" on:click=move |_| audio::say(&full)>"🔊"<small>"Aide"</small></button>
                </div>
            }
            .into_any()
        }
        Prompt::Big(s) => view! { <div class="big-emoji pop-in">{s}</div> }.into_any(),
        Prompt::Count { n, emoji } => view! {
            <div class="count-box" class:many={n > 6}>
                {(0..n).map(|i| view! { <span class="item" style:animation-delay=format!("{}ms", i * 60)>{emoji}</span> }).collect_view()}
            </div>
        }
        .into_any(),
        Prompt::Op { a, b, plus, emoji, items } => {
            if items {
                let group = |n: u32, crossed_from: u32| {
                    (0..n)
                        .map(|i| view! { <span class="item" class:crossed={i >= crossed_from}>{emoji}</span> })
                        .collect_view()
                };
                if plus {
                    view! {
                        <div class="op-box">
                            <div class="group">{group(a, u32::MAX)}</div>
                            <div class="op-sign">"+"</div>
                            <div class="group">{group(b, u32::MAX)}</div>
                            <div class="op-sign">"= ?"</div>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="op-box">
                            <div class="group">{group(a, a - b)}</div>
                            <div class="op-text">{format!("{a} − {b} = ?")}</div>
                        </div>
                    }
                    .into_any()
                }
            } else {
                let sign = if plus { "+" } else { "−" };
                view! { <div class="op-big">{format!("{a} {sign} {b} = ?")}</div> }.into_any()
            }
        }
    }
}

#[component]
fn QuizRun(
    make: fn(u8) -> Q,
    total: usize,
    on_done: Callback<u32>,
    #[prop(optional)] icon: Option<&'static str>,
) -> impl IntoView {
    let app = app();
    let alive = Alive::new();
    let level = app.level();
    let q = RwSignal::new(make(level));
    let idx = RwSignal::new(0usize);
    let wrong = RwSignal::new(Vec::<usize>::new());
    let solved = RwSignal::new(false);
    let correct = RwSignal::new(0u32);

    let say_q = move || audio::say(&q.with_untracked(|q| q.say.clone()));
    {
        let alive = alive.clone();
        set_timeout(
            move || {
                if alive.get() {
                    say_q()
                }
            },
            Duration::from_millis(350),
        );
    }

    let next = move || {
        let n = idx.get_untracked() + 1;
        if n >= total {
            on_done.run(correct.get_untracked());
            return;
        }
        let prev = q.get_untracked();
        let mut nq = make(level);
        for _ in 0..8 {
            if nq.prompt != prev.prompt || nq.say != prev.say {
                break;
            }
            nq = make(level);
        }
        idx.set(n);
        q.set(nq);
        wrong.set(Vec::new());
        solved.set(false);
        say_q();
    };

    let choose = move |i: usize| {
        if solved.get_untracked() || wrong.with_untracked(|w| w.contains(&i)) {
            return;
        }
        let (ans, domain) = q.with_untracked(|q| (q.answer, q.domain));
        let first = wrong.with_untracked(|w| w.is_empty());
        if i == ans {
            audio::good();
            solved.set(true);
            if first {
                correct.update(|c| *c += 1);
                app.add_stars(1);
                app.record(domain, true);
            }
            audio::say(pick(&["Bravo !", "Super !", "Génial !", "Bien joué !", "Oui !"]));
            let alive = alive.clone();
            set_timeout(
                move || {
                    if alive.get() {
                        next()
                    }
                },
                Duration::from_millis(1300),
            );
        } else {
            audio::bad();
            if first {
                app.record(domain, false);
            }
            wrong.update(|w| w.push(i));
            audio::say("Essaie encore !");
        }
    };

    let colors = ["blue", "pink", "yellow", "green"];

    view! {
        <div class="screen quiz">
            <TopBar>
                <div class="q-count">
                    {icon.map(|i| view! { <span>{i}" "</span> })}
                    "Question " {move || idx.get() + 1} " / " {total}
                </div>
            </TopBar>
            <div class="progress">
                <div
                    class="progress-fill"
                    style:width=move || {
                        let done = idx.get() as f64 + if solved.get() { 1.0 } else { 0.0 };
                        format!("{}%", done / total as f64 * 100.0)
                    }
                ></div>
            </div>
            <Cat text=Signal::derive(move || q.with(|q| q.ask.clone())) happy=Signal::derive(move || solved.get())/>
            <div class="prompt-zone">
                {move || {
                    let qq = q.get();
                    render_prompt(qq.prompt, qq.say)
                }}
                <button class="round repeat" on:click=move |_| say_q() aria-label="Répéter">"🔊"</button>
            </div>
            {move || {
                let qq = q.get();
                let n = qq.choices.len();
                let answer = qq.answer;
                let choose = choose.clone();
                view! {
                    <div class="choices" class:wide=qq.wide class:duo=n == 2>
                        {qq
                            .choices
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                let choose = choose.clone();
                                let color = if qq.wide { colors[i % 4] } else { "" };
                                view! {
                                    <button
                                        class=format!("choice {color}")
                                        class:ok=move || solved.get() && i == answer
                                        class:bad=move || wrong.with(|w| w.contains(&i))
                                        class:dim=move || solved.get() && i != answer
                                        on:click=move |_| choose(i)
                                    >
                                        {choice_label(c)}
                                        {move || (solved.get() && i == answer).then(|| view! { <span class="ok-star">"⭐"</span> })}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                }
            }}
        </div>
    }
}

/// Reading / letters / math: rounds of 10 questions.
#[component]
pub fn Learn(game: Game) -> impl IntoView {
    let make: fn(u8) -> Q = match game {
        Game::Reading => questions::reading,
        Game::Letters => questions::letters,
        _ => questions::math,
    };
    let done = RwSignal::new(None::<u32>);
    let on_done = Callback::new(move |c: u32| done.set(Some(c)));
    move || {
        let d = done.get();
        untrack(move || match d {
            None => view! { <QuizRun make=make total=10 on_done=on_done icon=game.emoji()/> }.into_any(),
            Some(c) => {
                let title = if c >= 8 { "Bravo, champion !" } else if c >= 5 { "Bien joué !" } else { "Tu progresses !" };
                view! {
                    <div class="screen">
                        <TopBar/>
                        <Finish
                            title=title
                            detail=format!("{c} bonnes réponses du premier coup sur 10")
                            stars=(c + 1) / 2
                            on_again=Callback::new(move |_| done.set(None))
                        />
                    </div>
                }
                .into_any()
            }
        })
    }
}

/// The wall of questions before an action game.
#[component]
pub fn Gate(game: Game) -> impl IntoView {
    let app = app();
    let total = app.settings.with_untracked(|s| s.gate_questions.max(1)) as usize;
    let on_done = Callback::new(move |_| {
        let _ = app.screen.try_set(Screen::Play(game));
    });
    view! { <QuizRun make=questions::mixed total=total on_done=on_done icon=game.emoji()/> }
}
