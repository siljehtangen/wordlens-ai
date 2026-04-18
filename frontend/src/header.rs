use leptos::prelude::*;

use crate::icons::{icon_eye, icon_info};
use crate::types::Lens;

#[component]
pub fn Header(
    active_lens: ReadSignal<Lens>,
    set_show_info: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="flex items-center gap-3 px-5 py-4 border-b shrink-0 glass-panel"
            style="border-color:var(--bot-border);background:var(--bg-secondary)">
            <div class="w-10 h-10 rounded-xl flex items-center justify-center text-white shrink-0"
                style="background:linear-gradient(135deg,var(--accent) 0%,var(--accent-bright) 100%);box-shadow:0 4px 16px var(--accent-glow)">
                {icon_eye()}
            </div>
            <div class="flex-1 min-w-0">
                <h1 class="text-base font-bold tracking-tight leading-none"
                    style="color:var(--accent)">
                    "WordLens AI"
                </h1>
                <p class="text-[0.68rem] mt-0.5 leading-none"
                    style="color:var(--text-secondary)">
                    {move || format!("{} lens · {}", active_lens.get().label(), active_lens.get().tagline())}
                </p>
            </div>
            <span class="text-[0.6rem] font-bold tracking-widest px-2.5 py-1 rounded-full uppercase shrink-0"
                style="background:var(--accent-light);color:var(--accent)">
                "AI"
            </span>
            <button
                class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 border transition-all duration-200 hover:scale-105 active:scale-95"
                style="color:var(--text-muted);border-color:var(--bot-border);background:transparent"
                on:click=move |_| set_show_info.set(true)
                aria-label="About WordLens AI"
                title="About WordLens AI"
            >
                {icon_info()}
            </button>
        </header>
    }
}
