use leptos::prelude::*;

use crate::icons::{icon_refresh, icon_send, icon_trash, lens_icon};
use crate::types::{Lens, Message, Role};

#[component]
pub fn InputBar(
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    active_lens: ReadSignal<Lens>,
    loading: ReadSignal<bool>,
    messages_per_lens: [RwSignal<Vec<Message>>; 5],
    on_send: impl Fn() + 'static,
    on_regenerate: impl Fn() + 'static,
) -> impl IntoView {
    let has_responses =
        move || messages_per_lens[active_lens.get().index()].get().iter().any(|m| m.role == Role::Assistant);

    view! {
        <footer class="px-4 pb-6 pt-3.5 border-t shrink-0 flex flex-col gap-2.5 glass-panel"
            style="background:var(--bg-secondary);border-color:var(--bot-border)">

            <div class="flex items-center gap-2.5 border rounded-2xl pl-4 pr-1.5 py-2 transition-colors duration-200 input-ring"
                style="background:var(--input-bg);border-color:var(--input-border)">

                // Active lens chip
                <div class="flex items-center gap-1.5 text-[0.68rem] font-bold rounded-full px-2.5 py-1 shrink-0 select-none"
                    style="color:var(--badge-text);background:var(--badge-bg)">
                    {move || lens_icon(active_lens.get())}
                    <span>{move || active_lens.get().label()}</span>
                </div>

                <input
                    class="flex-1 bg-transparent border-none outline-none text-[0.91rem] placeholder:opacity-40 min-w-0"
                    style="color:var(--text-primary)"
                    type="text"
                    placeholder="Type a word or concept…"
                    prop:value=move || input.get()
                    on:input=move |e| {
                        use wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value();
                        set_input.set(val);
                    }
                    on:keydown=move |e| {
                        if e.key() == "Enter" && !e.shift_key() {
                            e.prevent_default();
                            on_send();
                        }
                    }
                    disabled=move || loading.get()
                    aria-label="Word or concept input"
                />

                // Send button
                <button
                    class="w-10 h-10 rounded-xl text-white flex items-center justify-center shrink-0 transition-all duration-150 hover:scale-105 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed"
                    style="background:linear-gradient(135deg,var(--accent) 0%,var(--accent-bright) 100%);box-shadow:0 2px 10px var(--accent-glow)"
                    on:click=move |_| on_send()
                    disabled=move || loading.get() || input.get().trim().is_empty()
                    aria-label="Send"
                >
                    {move || if loading.get() {
                        view! { <span class="spinner" aria-hidden="true"/> }.into_any()
                    } else {
                        icon_send().into_any()
                    }}
                </button>
            </div>

            // Keyboard hint
            <p class="text-[0.63rem] text-right select-none -mt-1"
                style="color:var(--text-muted)">
                "Press Enter to send"
            </p>

            // Secondary actions
            {move || has_responses().then(|| view! {
                <div class="flex gap-2 justify-end">
                    <button
                        class="flex items-center gap-1.5 text-[0.72rem] font-medium px-3 py-1.5 rounded-full border hover:opacity-80 transition-all duration-200 disabled:opacity-40"
                        style="color:var(--text-secondary);border-color:var(--bot-border)"
                        on:click=move |_| on_regenerate()
                        disabled=move || loading.get()
                    >
                        {icon_refresh()}
                        "Regenerate"
                    </button>
                    <button
                        class="flex items-center gap-1.5 text-[0.72rem] font-medium px-3 py-1.5 rounded-full border hover:bg-red-500/10 hover:border-red-500/20 hover:text-red-400 transition-all duration-200"
                        style="color:var(--text-secondary);border-color:var(--bot-border)"
                        on:click=move |_| messages_per_lens[active_lens.get_untracked().index()].update(|v| v.clear())
                    >
                        {icon_trash()}
                        "Clear"
                    </button>
                </div>
            })}
        </footer>
    }
}
