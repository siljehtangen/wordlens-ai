import {
  component$,
  useSignal,
  useStore,
  $,
  useVisibleTask$,
  noSerialize,
  type NoSerialize,
} from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";

// ── Types ────────────────────────────────────────────────────────────────────

export type Lens = "simple" | "learning" | "game" | "cyberpunk" | "poetic";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  lens?: Lens;
  streaming?: boolean;
}

// ── Lens catalogue ────────────────────────────────────────────────────────────

const LENSES: { id: Lens; label: string; emoji: string; tagline: string }[] = [
  {
    id: "simple",
    label: "Simple",
    emoji: "📚",
    tagline: "Clear & easy",
  },
  {
    id: "learning",
    label: "Learning",
    emoji: "🧠",
    tagline: "Deep & structured",
  },
  {
    id: "game",
    label: "Game",
    emoji: "🎮",
    tagline: "Interactive & fun",
  },
  {
    id: "cyberpunk",
    label: "Cyberpunk",
    emoji: "🏙️",
    tagline: "Futuristic & dark",
  },
  {
    id: "poetic",
    label: "Poetic",
    emoji: "📖",
    tagline: "Metaphorical & beautiful",
  },
];

// ── Component ─────────────────────────────────────────────────────────────────

export default component$(() => {
  const input = useSignal("");
  const activeLens = useSignal<Lens>("simple");
  const loading = useSignal(false);
  const messagesEnd = useSignal<Element>();
  const chatArea = useSignal<Element>();

  const messages = useStore<{ list: Message[]; streamingId: string | null }>({
    list: [],
    streamingId: null,
  });

  // Keep an EventSource reference so we can close it on cleanup.
  const evtSrcHolder = useStore<{ ref: NoSerialize<EventSource> | null }>({
    ref: null,
  });

  // Auto-scroll whenever the message list grows or streaming content changes.
  useVisibleTask$(({ track }) => {
    track(() => messages.list.length);
    track(() =>
      messages.list.find((m) => m.streaming)?.content
    );
    messagesEnd.value?.scrollIntoView({ behavior: "smooth", block: "end" });
  });

  // ── Send message ─────────────────────────────────────────────────────────

  const sendMessage = $(async () => {
    const word = input.value.trim();
    if (!word || loading.value) return;

    // Close any previous stream.
    if (evtSrcHolder.ref) {
      evtSrcHolder.ref.close();
      evtSrcHolder.ref = null;
    }

    const baseId = crypto.randomUUID();
    messages.list.push({ id: `${baseId}-user`, role: "user", content: word });
    input.value = "";
    loading.value = true;

    const replyId = `${baseId}-reply`;

    try {
      // Use the streaming endpoint for a token-by-token experience.
      const resp = await fetch("/api/explain", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          word,
          lens: activeLens.value,
          stream: false, // switch to true when Ollama streaming is confirmed working
        }),
      });

      if (!resp.ok) {
        let msg = `Server error ${resp.status}`;
        try {
          const err = await resp.json();
          msg = err.error ?? msg;
        } catch { /* ignore */ }
        messages.list.push({
          id: replyId,
          role: "assistant",
          content: msg,
          lens: activeLens.value,
        });
        return;
      }

      const data = await resp.json();
      messages.list.push({
        id: replyId,
        role: "assistant",
        content: data.explanation,
        lens: activeLens.value,
      });
    } catch (err) {
      messages.list.push({
        id: replyId,
        role: "assistant",
        content:
          "Could not reach the WordLens backend. Is `cargo run` running on port 8080?",
        lens: activeLens.value,
      });
    } finally {
      loading.value = false;
    }
  });

  // ── Regenerate last response ──────────────────────────────────────────────

  const regenerate = $(async () => {
    // Find the last user message and resend it.
    const lastUser = [...messages.list].reverse().find((m) => m.role === "user");
    if (!lastUser) return;

    // Remove everything after (and including) the last assistant reply.
    const lastUserIdx = messages.list.lastIndexOf(lastUser);
    messages.list.splice(lastUserIdx + 1);

    input.value = lastUser.content;
    await sendMessage();
  });

  // ── Helpers ───────────────────────────────────────────────────────────────

  const lensInfo = (id?: Lens) => LENSES.find((l) => l.id === id);

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class={["app", `lens-${activeLens.value}`]}>
      {/* ── Header ─────────────────────────────────────── */}
      <header class="header">
        <div class="header-inner">
          <div class="logo">
            <span class="logo-icon" aria-hidden>🔍</span>
            <span class="logo-text">WordLens AI</span>
          </div>
          <p class="tagline">
            Understand anything through multiple perspectives
          </p>
        </div>
      </header>

      {/* ── Lens selector ──────────────────────────────── */}
      <nav class="lens-bar" aria-label="Select lens">
        {LENSES.map((lens) => (
          <button
            key={lens.id}
            class={["lens-btn", activeLens.value === lens.id ? "active" : ""]}
            onClick$={() => {
              activeLens.value = lens.id;
            }}
            aria-pressed={activeLens.value === lens.id}
            title={lens.tagline}
          >
            <span class="lens-emoji" aria-hidden>
              {lens.emoji}
            </span>
            <span class="lens-label">{lens.label}</span>
            <span class="lens-tagline">{lens.tagline}</span>
          </button>
        ))}
      </nav>

      {/* ── Chat area ──────────────────────────────────── */}
      <main class="chat-area" ref={chatArea}>
        {messages.list.length === 0 && (
          <div class="empty-state">
            <div class="empty-icon">✨</div>
            <p class="empty-title">Enter any word, concept, or idea</p>
            <p class="empty-examples">
              Try:{" "}
              {["entropy", "democracy", "recursion", "love", "gravity"].map(
                (ex, i) => (
                  <>
                    {i > 0 && ", "}
                    <button
                      key={ex}
                      class="example-chip"
                      onClick$={() => {
                        input.value = ex;
                      }}
                    >
                      {ex}
                    </button>
                  </>
                )
              )}
            </p>
          </div>
        )}

        {messages.list.map((msg) => {
          const info = lensInfo(msg.lens);
          return (
            <div
              key={msg.id}
              class={[
                "message",
                `message-${msg.role}`,
                msg.lens ? `msg-lens-${msg.lens}` : "",
                msg.streaming ? "streaming" : "",
              ]}
            >
              {msg.role === "assistant" && info && (
                <div class="msg-lens-badge">
                  <span aria-hidden>{info.emoji}</span> {info.label}
                </div>
              )}
              <div class="msg-content">{msg.content}</div>
              {msg.streaming && (
                <span class="cursor" aria-hidden>
                  ▌
                </span>
              )}
            </div>
          );
        })}

        {loading.value && (
          <div class="message message-assistant loading-msg">
            <div class="typing-dots">
              <span />
              <span />
              <span />
            </div>
          </div>
        )}

        <div ref={messagesEnd} class="scroll-anchor" />
      </main>

      {/* ── Input bar ──────────────────────────────────── */}
      <footer class="input-bar">
        <div class="input-inner">
          <div class="active-lens-chip">
            <span aria-hidden>{lensInfo(activeLens.value)?.emoji}</span>
            {lensInfo(activeLens.value)?.label}
          </div>
          <input
            class="chat-input"
            type="text"
            placeholder="Enter a word or concept…"
            value={input.value}
            onInput$={(e) => {
              input.value = (e.target as HTMLInputElement).value;
            }}
            onKeyDown$={(e) => {
              if (e.key === "Enter" && !e.shiftKey) sendMessage();
            }}
            disabled={loading.value}
            aria-label="Word or concept input"
          />
          <button
            class="send-btn"
            onClick$={sendMessage}
            disabled={loading.value || !input.value.trim()}
            aria-label="Send"
          >
            {loading.value ? (
              <span class="spinner" aria-hidden />
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                width="20"
                height="20"
                aria-hidden
              >
                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z" />
              </svg>
            )}
          </button>
        </div>

        {messages.list.some((m) => m.role === "assistant") && (
          <div class="input-actions">
            <button class="action-btn" onClick$={regenerate} disabled={loading.value}>
              ↺ Regenerate
            </button>
            <button
              class="action-btn"
              onClick$={() => {
                messages.list.splice(0, messages.list.length);
              }}
            >
              ✕ Clear
            </button>
          </div>
        )}
      </footer>
    </div>
  );
});

// ── Document head ─────────────────────────────────────────────────────────────

export const head: DocumentHead = {
  title: "WordLens AI",
  meta: [
    {
      name: "description",
      content:
        "Understand any word, concept, or idea through multiple AI-powered perspectives.",
    },
    { property: "og:title", content: "WordLens AI" },
    {
      property: "og:description",
      content: "Multi-perspective AI explanations powered by Llama 3.",
    },
  ],
};
