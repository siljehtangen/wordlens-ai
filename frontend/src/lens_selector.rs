use leptos::prelude::*;

use crate::icons::lens_icon;
use crate::types::Lens;

#[component]
pub fn LensSelector(
    active_lens: ReadSignal<Lens>,
    set_active_lens: WriteSignal<Lens>,
) -> impl IntoView {
    view! {
        <nav class="flex gap-2 px-4 py-3 border-b overflow-x-auto scrollbar-hide shrink-0"
            style="background:var(--bg-secondary);border-color:var(--bot-border)"
            aria-label="Select lens">
            {Lens::all().iter().map(|&lens| {
                view! {
                    <button
                        class=move || {
                            let active = active_lens.get() == lens;
                            let base = "flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs font-semibold whitespace-nowrap shrink-0 border transition-all duration-200";
                            if active {
                                format!("{base} text-white border-transparent scale-[1.02]")
                            } else {
                                format!("{base} hover:scale-[1.02] hover:-translate-y-px")
                            }
                        }
                        style=move || {
                            if active_lens.get() == lens {
                                "background:var(--accent);color:#fff;border-color:transparent;box-shadow:0 2px 12px var(--accent-glow)".to_string()
                            } else {
                                "color:var(--text-secondary);border-color:var(--bot-border)".to_string()
                            }
                        }
                        on:click=move |_| set_active_lens.set(lens)
                        aria-pressed=move || (active_lens.get() == lens).to_string()
                        title=lens.tagline()
                    >
                        {lens_icon(lens)}
                        <span>{lens.label()}</span>
                    </button>
                }
            }).collect_view()}
        </nav>
    }
}
