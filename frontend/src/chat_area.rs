use leptos::prelude::*;

use crate::icons::{lens_icon, icon_sparkles};
use crate::types::{Lens, Message, Role};

#[component]
pub fn ChatArea(
    active_lens: ReadSignal<Lens>,
    messages_per_lens: [RwSignal<Vec<Message>>; 5],
    loading: ReadSignal<bool>,
    set_input: WriteSignal<String>,
    messages_end: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    view! {
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
    }
}
