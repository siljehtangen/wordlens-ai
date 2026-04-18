use leptos::prelude::*;
use lucide_leptos::{
    BookOpen, Cpu, Eye, Feather, Gamepad2, GraduationCap, Info, RefreshCw, Send, Sparkles,
    Trash2, X,
};

use crate::types::Lens;

// ── Generic icons ─────────────────────────────────────────────────────────────

pub fn icon_eye() -> impl IntoView {
    view! { <Eye size=17 /> }
}

pub fn icon_send() -> impl IntoView {
    view! { <Send size=15 /> }
}

pub fn icon_refresh() -> impl IntoView {
    view! { <RefreshCw size=12 /> }
}

pub fn icon_trash() -> impl IntoView {
    view! { <Trash2 size=12 /> }
}

pub fn icon_info() -> impl IntoView {
    view! { <Info size=16 /> }
}

pub fn icon_close() -> impl IntoView {
    view! { <X size=16 /> }
}

pub fn icon_sparkles() -> impl IntoView {
    view! { <Sparkles size=40 stroke_width=1.5 /> }
}

// ── Per-lens icons ────────────────────────────────────────────────────────────

pub fn lens_icon(lens: Lens) -> impl IntoView {
    match lens {
        Lens::Simple    => view! { <BookOpen size=15 /> }.into_any(),
        Lens::Learning  => view! { <GraduationCap size=15 /> }.into_any(),
        Lens::Game      => view! { <Gamepad2 size=15 /> }.into_any(),
        Lens::Cyberpunk => view! { <Cpu size=15 /> }.into_any(),
        Lens::Poetic    => view! { <Feather size=15 /> }.into_any(),
    }
}
