use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Settings {
    pub name: String,
    /// 1 (4 ans) ..= 5 (7 ans)
    pub level: u8,
    /// Number of questions before an action game (0 = none).
    pub gate_questions: u8,
    /// Daily play time limit in minutes (0 = no limit).
    pub daily_limit_min: u32,
    pub sound: bool,
    /// Ids of games hidden from the home screen.
    pub hidden: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            name: String::new(),
            level: 2,
            gate_questions: 3,
            daily_limit_min: 0,
            sound: true,
            hidden: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default)]
pub struct Stats {
    pub stars: u32,
    /// "YYYY-MM-DD" -> seconds played
    pub days: BTreeMap<String, u32>,
    pub plays: BTreeMap<String, u32>,
    pub ok: BTreeMap<String, u32>,
    pub ko: BTreeMap<String, u32>,
}

impl Stats {
    pub fn today_seconds(&self) -> u32 {
        self.days.get(&crate::util::today()).copied().unwrap_or(0)
    }
}

const SETTINGS_KEY: &str = "minou.settings";
const STATS_KEY: &str = "minou.stats";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load<T: for<'a> Deserialize<'a> + Default>(key: &str) -> T {
    storage()
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|txt| serde_json::from_str(&txt).ok())
        .unwrap_or_default()
}

fn save<T: Serialize>(key: &str, v: &T) {
    if let (Some(s), Ok(txt)) = (storage(), serde_json::to_string(v)) {
        let _ = s.set_item(key, &txt);
    }
}

pub fn load_settings() -> Settings {
    let mut s: Settings = load(SETTINGS_KEY);
    s.level = s.level.clamp(1, 5);
    s
}
pub fn save_settings(s: &Settings) {
    save(SETTINGS_KEY, s)
}
pub fn load_stats() -> Stats {
    load(STATS_KEY)
}
pub fn save_stats(s: &Stats) {
    save(STATS_KEY, s)
}
