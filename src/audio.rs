use std::cell::{Cell, RefCell};
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, OscillatorType, SpeechSynthesisUtterance, SpeechSynthesisVoice};

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(true) };
    static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
}

pub fn set_enabled(b: bool) {
    ENABLED.with(|e| e.set(b));
    if !b {
        stop_speech();
    }
}

fn enabled() -> bool {
    ENABLED.with(|e| e.get())
}

/// Must be called from a user gesture at least once (iOS).
pub fn unlock() {
    with_ctx(|_| {});
}

fn with_ctx(f: impl FnOnce(&AudioContext)) {
    CTX.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            *c = AudioContext::new().ok();
        }
        if let Some(ctx) = c.as_ref() {
            if ctx.state() != web_sys::AudioContextState::Running {
                let _ = ctx.resume();
            }
            f(ctx);
        }
    });
}

fn french_voice(s: &web_sys::SpeechSynthesis) -> Option<SpeechSynthesisVoice> {
    let voices: Vec<SpeechSynthesisVoice> = s
        .get_voices()
        .iter()
        .filter_map(|v| v.dyn_into::<SpeechSynthesisVoice>().ok())
        .filter(|v| v.lang().to_lowercase().starts_with("fr"))
        .collect();
    let fr_fr = |v: &&SpeechSynthesisVoice| v.lang().to_lowercase().replace('_', "-") == "fr-fr";
    voices
        .iter()
        .filter(fr_fr)
        .find(|v| ["Audrey", "Amélie", "Marie", "Thomas"].iter().any(|n| v.name().contains(n)))
        .or_else(|| voices.iter().find(fr_fr))
        .or_else(|| voices.first())
        .cloned()
}

pub fn say(text: &str) {
    if !enabled() {
        return;
    }
    let Some(w) = web_sys::window() else { return };
    let Ok(s) = w.speech_synthesis() else { return };
    s.cancel();
    if let Ok(u) = SpeechSynthesisUtterance::new_with_text(text) {
        u.set_lang("fr-FR");
        u.set_rate(0.9);
        u.set_pitch(1.1);
        if let Some(v) = french_voice(&s) {
            u.set_voice(Some(&v));
        }
        s.speak(&u);
    }
}

pub fn stop_speech() {
    if let Some(Ok(s)) = web_sys::window().map(|w| w.speech_synthesis()) {
        s.cancel();
    }
}

/// Play notes: (frequency Hz, start s, duration s)
fn notes(list: &[(f32, f64, f64)], kind: OscillatorType, vol: f32) {
    if !enabled() {
        return;
    }
    with_ctx(|ctx| {
        let t0 = ctx.current_time();
        for &(freq, start, dur) in list {
            let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) else { return };
            osc.set_type(kind);
            osc.frequency().set_value(freq);
            let g = gain.gain();
            let _ = g.set_value_at_time(0.0001, t0 + start);
            let _ = g.exponential_ramp_to_value_at_time(vol, t0 + start + 0.015);
            let _ = g.exponential_ramp_to_value_at_time(0.0001, t0 + start + dur);
            let _ = osc.connect_with_audio_node(&gain);
            let _ = gain.connect_with_audio_node(&ctx.destination());
            let _ = osc.start_with_when(t0 + start);
            let _ = osc.stop_with_when(t0 + start + dur + 0.05);
        }
    });
}

pub fn good() {
    notes(&[(660.0, 0.0, 0.12), (990.0, 0.09, 0.22)], OscillatorType::Triangle, 0.25);
}
pub fn bad() {
    notes(&[(240.0, 0.0, 0.18), (190.0, 0.12, 0.25)], OscillatorType::Sine, 0.3);
}
pub fn pop() {
    notes(&[(520.0, 0.0, 0.07)], OscillatorType::Triangle, 0.2);
}
pub fn bump() {
    notes(&[(140.0, 0.0, 0.2)], OscillatorType::Square, 0.08);
}
pub fn win() {
    notes(
        &[(523.0, 0.0, 0.15), (659.0, 0.12, 0.15), (784.0, 0.24, 0.15), (1047.0, 0.36, 0.35)],
        OscillatorType::Triangle,
        0.25,
    );
}
