mod icons;
mod info_modal;
mod streaming;
mod types;

use leptos::prelude::*;
use leptos_meta::*;
use uuid::Uuid;

use icons::{
    icon_eye, icon_info, icon_refresh, icon_send, icon_sparkles, icon_trash, lens_icon,
};
use info_modal::InfoModal;
use streaming::stream_explain;
use types::{Lens, Message, Role};

// ── Main component ────────────────────────────────────────────────────────────

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let (input, set_input) = signal(String::new());
    let (active_lens, set_active_lens) = signal(Lens::Simple);
    let (loading, set_loading) = signal(false);
    let (show_info, set_show_info) = signal(false);
    let messages_per_lens: [RwSignal<Vec<Message>>; 5] =
        std::array::from_fn(|_| RwSignal::new(Vec::new()));

    // Auto-scroll anchor
    let messages_end = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        let _ = messages_per_lens[active_lens.get().index()].get();
        if let Some(el) = messages_end.get() {
            el.scroll_into_view();
        }
    });

    // ── send ──────────────────────────────────────────────────────────────────
    let send = move || {
        let word = input.get_untracked().trim().to_string();
        if word.is_empty() || loading.get_untracked() {
            return;
        }

        let base = Uuid::new_v4().to_string();
        let reply_id = format!("{base}-reply");

        let lens = active_lens.get_untracked();
        let messages = messages_per_lens[lens.index()];

        messages.update(|v| {
            v.push(Message {
                id: format!("{base}-user"),
                role: Role::User,
                content: RwSignal::new(word.clone()),
                lens: None,
                streaming: RwSignal::new(false),
            });
        });
        set_input.set(String::new());
        set_loading.set(true);

        leptos::task::spawn_local(stream_explain(
            word,
            lens,
            reply_id,
            messages,
            set_loading,
        ));
    };

    // ── regenerate ────────────────────────────────────────────────────────────
    let regenerate = move || {
        let lens = active_lens.get_untracked();
        let messages = messages_per_lens[lens.index()];
        let msgs = messages.get_untracked();
        let last_user = msgs.iter().rev().find(|m| m.role == Role::User).cloned();
        if let Some(u) = last_user {
            let idx = msgs.iter().rposition(|m| m.id == u.id).unwrap_or(0);
            messages.update(|v| v.truncate(idx + 1));
            let word = u.content.get_untracked();
            set_input.set(word.clone());
            let base = Uuid::new_v4().to_string();
            let reply_id = format!("{base}-reply");
            messages.update(|v| {
                if let Some(last) = v.last_mut() {
                    last.id = format!("{base}-user");
                }
            });
            set_loading.set(true);
            leptos::task::spawn_local(stream_explain(
                word,
                lens,
                reply_id,
                messages,
                set_loading,
            ));
        }
    };

    let has_responses =
        move || messages_per_lens[active_lens.get().index()].get().iter().any(|m| m.role == Role::Assistant);

    view! {
        <Title text="WordLens AI"/>

        <div class=move || format!(
            "app flex flex-col h-dvh w-full {}",
            active_lens.get().css_class()
        )>

            // ── Header ────────────────────────────────────────────────────────
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

            // ── Lens selector ─────────────────────────────────────────────────
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

            // ── Chat area ─────────────────────────────────────────────────────
            <main class="flex-1 overflow-y-auto px-4 py-5 flex flex-col gap-4 chat-scrollbar chat-inner"
                style="background:var(--bg-primary)">

                // Empty state
                {move || (messages_per_lens[active_lens.get().index()].get().is_empty()).then(|| view! {
                    <div class="flex-1 flex flex-col items-center justify-center gap-5 px-4 py-16 text-center select-none">
                        <div class="relative flex items-center justify-center w-20 h-20">
                            <div class="ambient-circle"/>
                            <div class="relative pulse-icon" style="color:var(--accent)">
                                {icon_sparkles()}
                            </div>
                        </div>
                        <div class="space-y-1.5">
                            <p class="font-bold text-[1.08rem] tracking-tight" style="color:var(--text-primary)">
                                "What would you like to understand?"
                            </p>
                            <p class="text-sm leading-relaxed" style="color:var(--text-secondary)">
                                "Type any word or concept — pick a lens to choose the style"
                            </p>
                        </div>
                        <div class="flex flex-wrap gap-2 justify-center mt-1">
                            {["entropy", "democracy", "recursion", "love", "gravity", "consciousness", "irony"].iter().map(|&ex| {
                                view! {
                                    <button
                                        class="px-3.5 py-1.5 rounded-full text-xs font-medium border hover:-translate-y-0.5 hover:shadow-sm transition-all duration-200 italic"
                                        style="background:var(--accent-light);color:var(--accent-bright);border-color:var(--bot-border)"
                                        on:click=move |_| set_input.set(ex.to_string())
                                    >
                                        {ex}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                })}

                // Messages
                <For
                    each=move || messages_per_lens[active_lens.get().index()].get()
                    key=|m| m.id.clone()
                    children=move |msg| {
                        let is_user = msg.role == Role::User;
                        let lens = msg.lens;
                        let content_sig = msg.content;
                        let streaming_sig = msg.streaming;

                        view! {
                            <div class=move || format!(
                                "flex flex-col max-w-[80%] msg-in {}",
                                if is_user { "self-end items-end" } else { "self-start items-start" }
                            )>
                                // Lens badge
                                {lens.map(|l| view! {
                                    <div class="flex items-center gap-1 mb-1.5">
                                        <span class="flex items-center gap-1.5 text-[0.65rem] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full"
                                            style="color:var(--badge-text);background:var(--badge-bg)">
                                            {lens_icon(l)}
                                            {l.label()}
                                        </span>
                                    </div>
                                })}

                                // Bubble
                                <div
                                    class=move || {
                                        let base = "px-4 py-3 leading-relaxed text-[0.94rem] break-words whitespace-pre-wrap";
                                        if is_user {
                                            format!("{base} rounded-2xl rounded-br-sm")
                                        } else {
                                            format!("{base} border rounded-2xl rounded-tl-sm")
                                        }
                                    }
                                    style=move || {
                                        if is_user {
                                            "background:linear-gradient(135deg,var(--user-bg) 0%,var(--accent-bright) 100%);color:var(--user-text);box-shadow:0 3px 14px var(--accent-glow)".to_string()
                                        } else {
                                            "background:var(--bot-bg);color:var(--bot-text);border-color:var(--bot-border);box-shadow:0 1px 8px rgba(0,0,0,0.07)".to_string()
                                        }
                                    }
                                >
                                    {move || content_sig.get()}
                                    {move || streaming_sig.get().then(|| view! {
                                        <span class="cursor ml-0.5" aria-hidden="true">"▌"</span>
                                    })}
                                </div>
                            </div>
                        }
                    }
                />

                // Typing indicator
                {move || loading.get().then(|| view! {
                    <div class="self-start msg-in">
                        <div class="typing-dots flex gap-2 px-4 py-3.5 border rounded-2xl rounded-tl-sm"
                            style="background:var(--bot-bg);border-color:var(--bot-border);box-shadow:0 1px 8px rgba(0,0,0,0.07)">
                            <span/>
                            <span/>
                            <span/>
                        </div>
                    </div>
                })}

                <div node_ref=messages_end class="h-px shrink-0"/>
            </main>

            // ── Input bar ─────────────────────────────────────────────────────
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
                                send();
                            }
                        }
                        disabled=move || loading.get()
                        aria-label="Word or concept input"
                    />

                    // Send button
                    <button
                        class="w-10 h-10 rounded-xl text-white flex items-center justify-center shrink-0 transition-all duration-150 hover:scale-105 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed"
                        style="background:linear-gradient(135deg,var(--accent) 0%,var(--accent-bright) 100%);box-shadow:0 2px 10px var(--accent-glow)"
                        on:click=move |_| send()
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
                            on:click=move |_| regenerate()
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

            // ── Info modal ────────────────────────────────────────────────────
            {move || show_info.get().then(|| view! {
                <InfoModal on_close=set_show_info />
            })}

        </div>
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
