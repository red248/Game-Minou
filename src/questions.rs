//! Question generators shared by the learning games and the wall of questions.
use crate::util::{pick, rand, range, shuffle};

#[derive(Clone, PartialEq, Debug)]
pub enum Prompt {
    None,
    /// Text to read; each word can be tapped to hear it.
    Read(String),
    /// A big emoji or letter.
    Big(String),
    /// `n` items to count.
    Count { n: u32, emoji: &'static str },
    /// a (+|-) b = ?  (with items when small)
    Op { a: u32, b: u32, plus: bool, emoji: &'static str, items: bool },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Q {
    /// Shown in the cat's bubble.
    pub ask: String,
    /// Spoken when the question appears / on 🔊.
    pub say: String,
    pub prompt: Prompt,
    /// A choice "A|B" is drawn as picture A with badge B.
    pub choices: Vec<String>,
    pub answer: usize,
    pub domain: &'static str,
    /// Show the choices as wide bars (numbers) instead of tiles.
    pub wide: bool,
}

fn with_choices(mut opts: Vec<String>, right: &str) -> (Vec<String>, usize) {
    opts.dedup();
    shuffle(&mut opts);
    let answer = opts.iter().position(|o| o == right).unwrap_or(0);
    (opts, answer)
}

// ---------------------------------------------------------------- Reading

pub const WORDS: &[(&str, &str)] = &[
    ("chat", "🐱"), ("chien", "🐶"), ("vache", "🐮"), ("lapin", "🐰"), ("lune", "🌙"),
    ("pomme", "🍎"), ("vélo", "🚲"), ("robot", "🤖"), ("lit", "🛏️"), ("moto", "🏍️"),
    ("rat", "🐀"), ("souris", "🐭"), ("poule", "🐔"), ("cochon", "🐷"),
    ("lion", "🦁"), ("soleil", "☀️"), ("bateau", "⛵"), ("maison", "🏠"), ("fleur", "🌸"),
    ("arbre", "🌳"), ("poisson", "🐟"), ("tortue", "🐢"), ("fusée", "🚀"), ("ballon", "⚽"),
    ("gâteau", "🎂"), ("livre", "📖"), ("canard", "🦆"), ("ours", "🐻"), ("train", "🚂"),
    ("étoile", "⭐"), ("banane", "🍌"), ("voiture", "🚗"), ("avion", "✈️"), ("cheval", "🐴"),
    ("escargot", "🐌"), ("papillon", "🦋"), ("clé", "🔑"), ("parapluie", "☂️"), ("tomate", "🍅"),
];
/// The first words are the easiest to decode.
const EASY_WORDS: usize = 12;

const SUBJECTS: &[(&str, &str, &str)] = &[
    // (singular, emoji, name used in plural phrases)
    ("Le chat", "🐱", "le chat"),
    ("Le chien", "🐶", "le chien"),
    ("La vache", "🐮", "la vache"),
    ("Maman", "👩", "maman"),
    ("Papa", "👨", "papa"),
    ("Le bébé", "👶", "le bébé"),
    ("Le lapin", "🐰", "le lapin"),
    ("Le robot", "🤖", "le robot"),
];

const ACTIONS: &[(&str, &str, &str)] = &[
    // (singular, plural, emoji)
    ("dort", "dorment", "💤"),
    ("mange", "mangent", "🍎"),
    ("lit", "lisent", "📖"),
    ("rit", "rient", "😂"),
    ("pleure", "pleurent", "😢"),
    ("chante", "chantent", "🎵"),
    ("nage", "nagent", "🌊"),
    ("joue au ballon", "jouent au ballon", "⚽"),
];

fn two_distinct(n: usize) -> (usize, usize) {
    let a = rand(n);
    let mut b = rand(n - 1);
    if b >= a {
        b += 1;
    }
    (a, b)
}

pub fn reading(level: u8) -> Q {
    let mode = match level {
        1 => 0,
        2 => rand(2),
        3 => 1,
        _ => 1 + rand(2),
    };
    match mode {
        0 => {
            let pool = if level == 1 { &WORDS[..EASY_WORDS] } else { WORDS };
            let mut idx: Vec<usize> = (0..pool.len()).collect();
            shuffle(&mut idx);
            let mut picked: Vec<(&str, &str)> = Vec::new();
            for i in idx {
                if !picked.iter().any(|p| p.1 == pool[i].1) {
                    picked.push(pool[i]);
                }
                if picked.len() == 4 {
                    break;
                }
            }
            let (word, emoji) = picked[0];
            let (choices, answer) = with_choices(picked.iter().map(|p| p.1.to_string()).collect(), emoji);
            Q {
                ask: "Quelle image va avec le mot ?".into(),
                say: "Lis le mot. Quelle image va avec le mot ?".into(),
                prompt: Prompt::Read(word.to_string()),
                choices,
                answer,
                domain: "lecture",
                wide: false,
            }
        }
        1 => {
            let (s1, s2) = two_distinct(SUBJECTS.len());
            let (a1, a2) = two_distinct(ACTIONS.len());
            let pic = |s: usize, a: usize| format!("{}|{}", SUBJECTS[s].1, ACTIONS[a].2);
            let right = pic(s1, a1);
            let opts = vec![right.clone(), pic(s1, a2), pic(s2, a1), pic(s2, a2)];
            let (choices, answer) = with_choices(opts, &right);
            Q {
                ask: "Quelle image va avec la phrase ?".into(),
                say: "Lis la phrase. Quelle image va avec la phrase ?".into(),
                prompt: Prompt::Read(format!("{} {}.", SUBJECTS[s1].0, ACTIONS[a1].0)),
                choices,
                answer,
                domain: "lecture",
                wide: false,
            }
        }
        _ => {
            // Two subjects together: "Le chat et le chien dorment."
            let mut s: Vec<usize> = (0..SUBJECTS.len()).collect();
            shuffle(&mut s);
            let (a1, a2) = two_distinct(ACTIONS.len());
            let pair = |x: usize, y: usize, a: usize| {
                format!("{}{}|{}", SUBJECTS[x].1, SUBJECTS[y].1, ACTIONS[a].2)
            };
            let right = pair(s[0], s[1], a1);
            let opts = vec![right.clone(), pair(s[0], s[2], a1), pair(s[0], s[1], a2), pair(s[3], s[1], a1)];
            let (choices, answer) = with_choices(opts, &right);
            Q {
                ask: "Quelle image va avec la phrase ?".into(),
                say: "Lis la phrase. Quelle image va avec la phrase ?".into(),
                prompt: Prompt::Read(format!("{} et {} {}.", SUBJECTS[s[0]].0, SUBJECTS[s[1]].2, ACTIONS[a1].1)),
                choices,
                answer,
                domain: "lecture",
                wide: false,
            }
        }
    }
}

// ---------------------------------------------------------------- Letters

pub fn letter_name(c: char) -> &'static str {
    match c.to_ascii_uppercase() {
        'A' => "a", 'B' => "bé", 'C' => "cé", 'D' => "dé", 'E' => "e", 'F' => "èf", 'G' => "jé",
        'H' => "ache", 'I' => "i", 'J' => "ji", 'K' => "ka", 'L' => "èl", 'M' => "èm", 'N' => "èn",
        'O' => "o", 'P' => "pé", 'Q' => "ku", 'R' => "èr", 'S' => "èss", 'T' => "té", 'U' => "u",
        'V' => "vé", 'W' => "double vé", 'X' => "iks", 'Y' => "i grec", 'Z' => "zèd", _ => "",
    }
}

fn letters_except(right: char, n: usize) -> Vec<char> {
    let mut v = vec![right];
    while v.len() < n {
        let c = (b'A' + rand(26) as u8) as char;
        if !v.contains(&c) {
            v.push(c);
        }
    }
    v
}

pub fn letters(level: u8) -> Q {
    let mode = match level {
        1 => 0,
        2 => rand(2),
        3 => 1 + rand(2),
        _ => 2,
    };
    match mode {
        0 => {
            let c = (b'A' + rand(26) as u8) as char;
            let opts: Vec<String> = letters_except(c, 4).iter().map(|c| c.to_string()).collect();
            let (choices, answer) = with_choices(opts, &c.to_string());
            Q {
                ask: format!("Trouve la lettre {c}"),
                say: format!("Trouve la lettre {}", letter_name(c)),
                prompt: Prompt::None,
                choices,
                answer,
                domain: "lettres",
                wide: false,
            }
        }
        1 => {
            let c = (b'A' + rand(26) as u8) as char;
            let opts: Vec<String> = letters_except(c, 4).iter().map(|c| c.to_ascii_lowercase().to_string()).collect();
            let (choices, answer) = with_choices(opts, &c.to_ascii_lowercase().to_string());
            Q {
                ask: "Trouve la même lettre en minuscule".into(),
                say: format!("Voici la lettre {}. Trouve la même lettre, en minuscule.", letter_name(c)),
                prompt: Prompt::Big(c.to_string()),
                choices,
                answer,
                domain: "lettres",
                wide: false,
            }
        }
        _ => {
            let pool: Vec<&(&str, &str)> = WORDS.iter().filter(|w| w.0.chars().next().unwrap().is_ascii()).collect();
            let (word, emoji) = *pick(&pool);
            let c = word.chars().next().unwrap().to_ascii_uppercase();
            let opts: Vec<String> = letters_except(c, 4).iter().map(|c| c.to_string()).collect();
            let (choices, answer) = with_choices(opts, &c.to_string());
            Q {
                ask: format!("Par quelle lettre commence « {word} » ?"),
                say: format!("Par quelle lettre commence le mot {word} ?"),
                prompt: Prompt::Big(emoji.to_string()),
                choices,
                answer,
                domain: "lettres",
                wide: false,
            }
        }
    }
}

// ---------------------------------------------------------------- Math

const FRUITS: &[&str] = &["🍎", "🍓", "🍊", "🍌", "🍐", "🍒", "🥕", "🍋", "🫐", "🍉"];

fn number_choices(right: u32, max: u32) -> (Vec<String>, usize) {
    let mut v = vec![right];
    let mut guard = 0;
    while v.len() < 3 && guard < 100 {
        guard += 1;
        let d = range(-3, 3);
        let n = right as i32 + d;
        if d != 0 && n >= 0 && n as u32 <= max + 2 && !v.contains(&(n as u32)) {
            v.push(n as u32);
        }
    }
    let opts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
    with_choices(opts, &right.to_string())
}

pub fn math(level: u8) -> Q {
    let emoji = pick(FRUITS);
    let count = |max: u32| {
        let n = range(1, max as i32) as u32;
        let (choices, answer) = number_choices(n, max);
        Q {
            ask: "Compte les fruits !".into(),
            say: "Compte les fruits ! Combien y en a-t-il ?".into(),
            prompt: Prompt::Count { n, emoji },
            choices,
            answer,
            domain: "calculs",
            wide: true,
        }
    };
    let op = |max: u32, allow_minus: bool, items: bool| {
        let plus = !allow_minus || rand(2) == 0;
        let (a, b) = if plus {
            let a = range(1, max as i32 - 1) as u32;
            let b = range(1, (max - a) as i32) as u32;
            (a, b)
        } else {
            let a = range(2, max as i32) as u32;
            let b = range(1, a as i32 - 1) as u32;
            (a, b)
        };
        let r = if plus { a + b } else { a - b };
        let (choices, answer) = number_choices(r, max);
        let word = if plus { "plus" } else { "moins" };
        Q {
            ask: format!("Combien font {a} {} {b} ?", if plus { "+" } else { "−" }),
            say: format!("Combien font {a} {word} {b} ?"),
            prompt: Prompt::Op { a, b, plus, emoji, items },
            choices,
            answer,
            domain: "calculs",
            wide: true,
        }
    };
    match level {
        1 => count(5),
        2 => {
            if rand(3) == 0 {
                op(5, false, true)
            } else {
                count(10)
            }
        }
        3 => {
            if rand(3) == 0 {
                count(10)
            } else {
                op(10, false, true)
            }
        }
        4 => op(10, true, true),
        _ => op(20, true, false),
    }
}

// ---------------------------------------------------------------- Logic (balance)

const THINGS: &[(&str, &str, u32)] = &[
    ("une plume", "🪶", 1),
    ("une cerise", "🍒", 2),
    ("une souris", "🐭", 3),
    ("une pomme", "🍎", 4),
    ("un chat", "🐱", 6),
    ("une pastèque", "🍉", 7),
    ("un chien", "🐶", 8),
    ("un cheval", "🐴", 10),
    ("une voiture", "🚗", 12),
    ("un éléphant", "🐘", 14),
];

pub fn balance() -> Q {
    let (i, j) = two_distinct(THINGS.len());
    let heavy = rand(2) == 0;
    let right = if (THINGS[i].2 > THINGS[j].2) == heavy { i } else { j };
    let word = if heavy { "le plus lourd" } else { "le plus léger" };
    let opts = vec![THINGS[i].1.to_string(), THINGS[j].1.to_string()];
    let (choices, answer) = with_choices(opts, THINGS[right].1);
    Q {
        ask: format!("Lequel est {word} ?"),
        say: format!("Lequel est {word} ?"),
        prompt: Prompt::Big("⚖️".into()),
        choices,
        answer,
        domain: "logique",
        wide: false,
    }
}

/// A mix for the wall of questions.
pub fn mixed(level: u8) -> Q {
    match rand(7) {
        0 | 1 => reading(level),
        2 | 3 => math(level),
        4 | 5 => letters(level),
        _ => balance(),
    }
}
