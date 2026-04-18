mod chat_area;
mod header;
mod icons;
mod info_modal;
mod input_bar;
mod lens_selector;
mod streaming;
mod types;

use leptos::prelude::*;
use leptos_meta::*;
use uuid::Uuid;

use chat_area::ChatArea;
use header::Header;
use info_modal::InfoModal;
use input_bar::InputBar;
use lens_selector::LensSelector;
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

    view! {
        <Title text="WordLens AI"/>

        <div class=move || format!(
            "app flex flex-col h-dvh w-full {}",
            active_lens.get().css_class()
        )>
            <Header active_lens set_show_info />
            <LensSelector active_lens set_active_lens />
            <ChatArea active_lens messages_per_lens loading set_input messages_end />
            <InputBar input set_input active_lens loading messages_per_lens on_send=send on_regenerate=regenerate />

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
