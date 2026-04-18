use futures::StreamExt;
use leptos::prelude::*;
use leptos_meta::*;
use serde_json::json;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Copy)]
enum Lens {
    Simple,
    Learning,
    Game,
    Cyberpunk,
    Poetic,
}

impl Lens {
    fn id(self) -> &'static str {
        match self {
            Lens::Simple => "simple",
            Lens::Learning => "learning",
            Lens::Game => "game",
            Lens::Cyberpunk => "cyberpunk",
            Lens::Poetic => "poetic",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Lens::Simple => "Simple",
            Lens::Learning => "Learning",
            Lens::Game => "Game",
            Lens::Cyberpunk => "Cyberpunk",
            Lens::Poetic => "Poetic",
        }
    }
    fn tagline(self) -> &'static str {
        match self {
            Lens::Simple => "Clear & easy",
            Lens::Learning => "Deep & structured",
            Lens::Game => "Interactive & fun",
            Lens::Cyberpunk => "Futuristic & dark",
            Lens::Poetic => "Metaphorical & beautiful",
        }
    }
    fn css_class(self) -> &'static str {
        match self {
            Lens::Simple => "lens-simple",
            Lens::Learning => "lens-learning",
            Lens::Game => "lens-game",
            Lens::Cyberpunk => "lens-cyberpunk",
            Lens::Poetic => "lens-poetic",
        }
    }
    fn all() -> &'static [Lens] {
        &[
            Lens::Simple,
            Lens::Learning,
            Lens::Game,
            Lens::Cyberpunk,
            Lens::Poetic,
        ]
    }
    fn index(self) -> usize {
        match self {
            Lens::Simple => 0,
            Lens::Learning => 1,
            Lens::Game => 2,
            Lens::Cyberpunk => 3,
            Lens::Poetic => 4,
        }
    }
}

#[derive(Clone, Debug)]
struct Message {
    id: String,
    role: Role,
    content: RwSignal<String>,
    lens: Option<Lens>,
    streaming: RwSignal<bool>,
}

#[derive(Clone, Debug, PartialEq)]
enum Role {
    User,
    Assistant,
}

// ── SVG Icons ─────────────────────────────────────────────────────────────────

fn icon_eye() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
            <circle cx="12" cy="12" r="3"/>
        </svg>
    }
}

fn icon_send() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2.5"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m22 2-7 20-4-9-9-4Z"/>
            <path d="M22 2 11 13"/>
        </svg>
    }
}

fn icon_refresh() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
            <path d="M21 3v5h-5"/>
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
            <path d="M8 16H3v5"/>
        </svg>
    }
}

fn icon_trash() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 6h18"/>
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
            <line x1="10" x2="10" y1="11" y2="17"/>
            <line x1="14" x2="14" y1="11" y2="17"/>
        </svg>
    }
}

fn icon_info() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 16v-4"/>
            <path d="M12 8h.01"/>
        </svg>
    }
}

fn icon_close() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M18 6 6 18"/>
            <path d="m6 6 12 12"/>
        </svg>
    }
}

fn icon_sparkles() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
            <path d="M5 3v4"/><path d="M3 5h4"/>
            <path d="M19 17v4"/><path d="M17 19h4"/>
        </svg>
    }
}

fn lens_icon(lens: Lens) -> impl IntoView {
    match lens {
        Lens::Simple => view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
            </svg>
        }.into_any(),
        Lens::Learning => view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96-.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 1.98-3A2.5 2.5 0 0 1 9.5 2Z"/>
                <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96-.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-1.98-3A2.5 2.5 0 0 0 14.5 2Z"/>
            </svg>
        }.into_any(),
        Lens::Game => view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <line x1="6" x2="10" y1="12" y2="12"/>
                <line x1="8" x2="8" y1="10" y2="14"/>
                <line x1="15" x2="15.01" y1="13" y2="13"/>
                <line x1="18" x2="18.01" y1="11" y2="11"/>
                <rect width="20" height="12" x="2" y="6" rx="2"/>
            </svg>
        }.into_any(),
        Lens::Cyberpunk => view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="4" y="4" width="16" height="16" rx="2"/>
                <rect x="9" y="9" width="6" height="6"/>
                <path d="M15 2v2"/><path d="M15 20v2"/>
                <path d="M2 15h2"/><path d="M2 9h2"/>
                <path d="M20 15h2"/><path d="M20 9h2"/>
                <path d="M9 2v2"/><path d="M9 20v2"/>
            </svg>
        }.into_any(),
        Lens::Poetic => view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
                <line x1="16" x2="2" y1="8" y2="22"/>
                <line x1="17.5" x2="9" y1="15" y2="15"/>
            </svg>
        }.into_any(),
    }
}

// ── SSE streaming ─────────────────────────────────────────────────────────────

fn parse_sse_chunk(buffer: &mut String, new_data: &str) -> Vec<(String, bool)> {
    buffer.push_str(new_data);
    let mut results = Vec::new();

    while let Some(pos) = buffer.find("\n\n") {
        let block = buffer[..pos].to_string();
        *buffer = buffer[pos + 2..].to_string();

        let mut event_type = "message".to_string();
        let mut data = String::new();

        for line in block.lines() {
            if let Some(s) = line.strip_prefix("event: ") {
                event_type = s.trim().to_string();
            } else if let Some(s) = line.strip_prefix("data: ") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(s);
            }
        }

        if event_type == "done" {
            results.push((String::new(), true));
        } else if !data.is_empty() {
            results.push((data, false));
        }
    }
    results
}

async fn stream_explain(
    word: String,
    lens: Lens,
    reply_id: String,
    messages: RwSignal<Vec<Message>>,
    loading: WriteSignal<bool>,
) {
    // Add placeholder immediately so errors are visible in the chat
    let content_sig = RwSignal::new(String::new());
    let streaming_sig = RwSignal::new(true);
    messages.update(|v| {
        v.push(Message {
            id: reply_id.clone(),
            role: Role::Assistant,
            content: content_sig,
            lens: Some(lens),
            streaming: streaming_sig,
        });
    });
    loading.set(false);

    let body = json!({ "word": word, "lens": lens.id(), "stream": true }).to_string();

    let headers = Headers::new().unwrap();
    headers.set("Content-Type", "application/json").unwrap();

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));
    opts.set_headers(&headers);

    let request = match Request::new_with_str_and_init("/api/explain", &opts) {
        Ok(r) => r,
        Err(e) => {
            push_error(content_sig, streaming_sig, &format!("{e:?}"));
            return;
        }
    };

    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            finish_streaming(streaming_sig);
            return;
        }
    };

    let resp_val = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => {
            push_error(
                content_sig,
                streaming_sig,
                "Could not reach the WordLens backend. Is `cargo run` running on port 3001?",
            );
            web_sys::console::error_1(&e);
            return;
        }
    };

    let resp: Response = resp_val.unchecked_into();

    if !resp.ok() {
        push_error(
            content_sig,
            streaming_sig,
            &format!("Server error {}", resp.status()),
        );
        return;
    }

    let raw_body = match resp.body() {
        Some(b) => b,
        None => {
            finish_streaming(streaming_sig);
            return;
        }
    };

    let stream = wasm_streams::ReadableStream::from_raw(raw_body.unchecked_into());
    let mut stream = stream.into_stream();
    let mut buffer = String::new();

    'outer: while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(_) => break,
        };
        let arr = js_sys::Uint8Array::new(&chunk);
        let bytes = arr.to_vec();
        let text = String::from_utf8_lossy(&bytes);

        for (token, done) in parse_sse_chunk(&mut buffer, &text) {
            if done {
                break 'outer;
            }
            if !token.is_empty() {
                content_sig.update(|c| c.push_str(&token));
            }
        }
    }

    finish_streaming(streaming_sig);
}

fn push_error(content_sig: RwSignal<String>, streaming_sig: RwSignal<bool>, err: &str) {
    content_sig.set(err.to_string());
    streaming_sig.set(false);
}

fn finish_streaming(streaming_sig: RwSignal<bool>) {
    streaming_sig.set(false);
}

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
        let _ = messages_per_lens[active_lens.get().index()].get(); // track current lens messages
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
            // re-trigger send on next microtask
            let word = word;
            let base = Uuid::new_v4().to_string();
            let reply_id = format!("{base}-reply");
            messages.update(|v| {
                // replace the user message at the truncation point
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
                <div
                    class="fixed inset-0 z-50 flex items-center justify-center p-4"
                    style="background:rgba(0,0,0,0.55);backdrop-filter:blur(4px)"
                    on:click=move |e| {
                        use wasm_bindgen::JsCast;
                        if let Some(target) = e.target() {
                            if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                                if el.class_name().contains("inset-0") {
                                    set_show_info.set(false);
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
                            on:click=move |_| set_show_info.set(false)
                            aria-label="Close"
                        >
                            {icon_close()}
                        </button>

                        // Title
                        <h2 class="text-base font-bold mb-1" style="color:var(--accent)">
                            "About WordLens AI"
                        </h2>
                        <p class="text-sm leading-relaxed mb-5" style="color:var(--text-secondary)">
                            "WordLens AI helps you understand any word, concept, or idea by explaining it through different styles and perspectives — powered by a local AI model. Just type something and pick a lens."
                        </p>

                        // Lenses
                        <h3 class="text-[0.7rem] font-bold uppercase tracking-widest mb-3" style="color:var(--text-muted)">
                            "The Lenses"
                        </h3>
                        <ul class="flex flex-col gap-3">
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg" style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(Lens::Simple)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5" style="color:var(--text-primary)">"Simple"</p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">"Short, plain-language explanations. Great when you just want a quick, clear answer without jargon."</p>
                                </div>
                            </li>
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg" style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(Lens::Learning)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5" style="color:var(--text-primary)">"Learning"</p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">"Structured, educational breakdowns with context and examples. Perfect for studying or going deep on a topic."</p>
                                </div>
                            </li>
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg" style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(Lens::Game)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5" style="color:var(--text-primary)">"Game"</p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">"Explains concepts as if they were game mechanics — fun, energetic, and full of analogies from gaming culture."</p>
                                </div>
                            </li>
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg" style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(Lens::Cyberpunk)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5" style="color:var(--text-primary)">"Cyberpunk"</p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">"Dark, futuristic, and tech-noir. Concepts reframed through a dystopian sci-fi lens for a moody, immersive vibe."</p>
                                </div>
                            </li>
                            <li class="flex gap-3 items-start">
                                <span class="mt-0.5 shrink-0 p-1.5 rounded-lg" style="background:var(--accent-light);color:var(--accent)">
                                    {lens_icon(Lens::Poetic)}
                                </span>
                                <div>
                                    <p class="text-sm font-semibold leading-none mb-0.5" style="color:var(--text-primary)">"Poetic"</p>
                                    <p class="text-xs leading-relaxed" style="color:var(--text-secondary)">"Lyrical, metaphor-rich descriptions that paint a picture. Ideal when you want to feel the meaning, not just know it."</p>
                                </div>
                            </li>
                        </ul>
                    </div>
                </div>
            })}

        </div>
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
