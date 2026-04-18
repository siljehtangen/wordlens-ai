use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::icons::{icon_close, lens_icon};
use crate::types::Lens;

#[component]
pub fn InfoModal(on_close: WriteSignal<bool>) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4"
            style="background:rgba(0,0,0,0.55);backdrop-filter:blur(4px)"
            on:click=move |e| {
                if let Some(target) = e.target() {
                    if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                        if el.class_name().contains("inset-0") {
                            on_close.set(false);
                        }
                    }
                }
            }
        >
            <div
                class="relative w-full max-w-md rounded-2xl p-6 shadow-2xl overflow-y-auto max-h-[85dvh]"
                style="background:var(--bg-secondary);border:1px solid var(--bot-border)"
            >
                // Close button
                <button
                    class="absolute top-4 right-4 w-7 h-7 rounded-full flex items-center justify-center border hover:opacity-70 transition-opacity"
                    style="color:var(--text-muted);border-color:var(--bot-border)"
                    on:click=move |_| on_close.set(false)
                    aria-label="Close"
                >
                    {icon_close()}
                </button>

                // Title
                <h2 class="text-base font-bold mb-1" style="color:var(--accent)">
                    "About WordLens AI"
                </h2>
                <p class="text-sm leading-relaxed mb-5" style="color:var(--text-secondary)">
                    "WordLens AI helps you understand any word, concept, or idea by explaining it \
                     through different styles and perspectives — powered by a local AI model. \
                     Just type something and pick a lens."
                </p>

                // Lenses
                <h3 class="text-[0.7rem] font-bold uppercase tracking-widest mb-3" style="color:var(--text-muted)">
                    "The Lenses"
                </h3>
                <ul class="flex flex-col gap-3">
                    {Lens::all().iter().map(|&lens| {
                        let description = match lens {
                            Lens::Simple    => "Short, plain-language explanations. Great when you just want a quick, clear answer without jargon.",
                            Lens::Learning  => "Structured, educational breakdowns with context and examples. Perfect for studying or going deep on a topic.",
                            Lens::Game      => "Explains concepts as if they were game mechanics — fun, energetic, and full of analogies from gaming culture.",
                            Lens::Cyberpunk => "Dark, futuristic, and tech-noir. Concepts reframed through a dystopian sci-fi lens for a moody, immersive vibe.",
                            Lens::Poetic    => "Lyrical, metaphor-rich descriptions that paint a picture. Ideal when you want to feel the meaning, not just know it.",
                        };
                        view! {
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg"
                                    style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(lens)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5"
                                        style="color:var(--text-primary)">
                                        {lens.label()}
                                    </p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">
                                        {description}
                                    </p>
                                </div>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}
