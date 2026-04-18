use leptos::prelude::*;

// ── Lens ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Lens {
    Simple,
    Learning,
    Game,
    Cyberpunk,
    Poetic,
}

impl Lens {
    pub fn id(self) -> &'static str {
        match self {
            Lens::Simple => "simple",
            Lens::Learning => "learning",
            Lens::Game => "game",
            Lens::Cyberpunk => "cyberpunk",
            Lens::Poetic => "poetic",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Lens::Simple => "Simple",
            Lens::Learning => "Learning",
            Lens::Game => "Game",
            Lens::Cyberpunk => "Cyberpunk",
            Lens::Poetic => "Poetic",
        }
    }
    pub fn tagline(self) -> &'static str {
        match self {
            Lens::Simple => "Clear & easy",
            Lens::Learning => "Deep & structured",
            Lens::Game => "Interactive & fun",
            Lens::Cyberpunk => "Futuristic & dark",
            Lens::Poetic => "Metaphorical & beautiful",
        }
    }
    pub fn css_class(self) -> &'static str {
        match self {
            Lens::Simple => "lens-simple",
            Lens::Learning => "lens-learning",
            Lens::Game => "lens-game",
            Lens::Cyberpunk => "lens-cyberpunk",
            Lens::Poetic => "lens-poetic",
        }
    }
    pub fn all() -> &'static [Lens] {
        &[
            Lens::Simple,
            Lens::Learning,
            Lens::Game,
            Lens::Cyberpunk,
            Lens::Poetic,
        ]
    }
    pub fn index(self) -> usize {
        match self {
            Lens::Simple => 0,
            Lens::Learning => 1,
            Lens::Game => 2,
            Lens::Cyberpunk => 3,
            Lens::Poetic => 4,
        }
    }
}

// ── Message / Role ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: RwSignal<String>,
    pub lens: Option<Lens>,
    pub streaming: RwSignal<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    User,
    Assistant,
}
