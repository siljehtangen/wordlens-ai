use futures::StreamExt;
use leptos::prelude::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

use crate::types::{Lens, Message, Role};

// ── SSE parsing ───────────────────────────────────────────────────────────────

pub fn parse_sse_chunk(buffer: &mut String, new_data: &str) -> Vec<(String, bool)> {
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

// ── Streaming fetch ───────────────────────────────────────────────────────────

pub async fn stream_explain(
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_error(content_sig: RwSignal<String>, streaming_sig: RwSignal<bool>, err: &str) {
    content_sig.set(err.to_string());
    streaming_sig.set(false);
}

fn finish_streaming(streaming_sig: RwSignal<bool>) {
    streaming_sig.set(false);
}
