import {
  component$,
  useSignal,
  useStore,
  $,
  useVisibleTask$,
  type JSXOutput,
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

const IconBookOpen = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
    <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
  </svg>
);

const IconBrain = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96-.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 1.98-3A2.5 2.5 0 0 1 9.5 2Z"/>
    <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96-.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-1.98-3A2.5 2.5 0 0 0 14.5 2Z"/>
  </svg>
);

const IconGamepad2 = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <line x1="6" x2="10" y1="12" y2="12"/>
    <line x1="8" x2="8" y1="10" y2="14"/>
    <line x1="15" x2="15.01" y1="13" y2="13"/>
    <line x1="18" x2="18.01" y1="11" y2="11"/>
    <rect width="20" height="12" x="2" y="6" rx="2"/>
  </svg>
);

const IconCpu = () => (
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
);

const IconFeather = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
    <line x1="16" x2="2" y1="8" y2="22"/>
    <line x1="17.5" x2="9" y1="15" y2="15"/>
  </svg>
);

const LENS_ICONS: Record<Lens, () => JSXOutput> = {
  simple:    IconBookOpen,
  learning:  IconBrain,
  game:      IconGamepad2,
  cyberpunk: IconCpu,
  poetic:    IconFeather,
};

const LENSES: { id: Lens; label: string; tagline: string }[] = [
  { id: "simple",    label: "Simple",    tagline: "Clear & easy" },
  { id: "learning",  label: "Learning",  tagline: "Deep & structured" },
  { id: "game",      label: "Game",      tagline: "Interactive & fun" },
  { id: "cyberpunk", label: "Cyberpunk", tagline: "Futuristic & dark" },
  { id: "poetic",    label: "Poetic",    tagline: "Metaphorical & beautiful" },
];

// ── Lucide-style inline SVG icons ─────────────────────────────────────────────

const IconEye = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
    <circle cx="12" cy="12" r="3"/>
  </svg>
);

const IconSend = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2.5"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="m22 2-7 20-4-9-9-4Z"/>
    <path d="M22 2 11 13"/>
  </svg>
);

const IconRefreshCw = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
    <path d="M21 3v5h-5"/>
    <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
    <path d="M8 16H3v5"/>
  </svg>
);

const IconTrash2 = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3 6h18"/>
    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
    <line x1="10" x2="10" y1="11" y2="17"/>
    <line x1="14" x2="14" y1="11" y2="17"/>
  </svg>
);

const IconSparkles = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"
    fill="none" stroke="currentColor" stroke-width="1.5"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
    <path d="M5 3v4"/>
    <path d="M3 5h4"/>
    <path d="M19 17v4"/>
    <path d="M17 19h4"/>
  </svg>
);

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

  // Auto-scroll whenever message list grows or streaming content changes
  useVisibleTask$(({ track }) => {
    track(() => messages.list.length);
    track(() => messages.list.find((m) => m.streaming)?.content);
    messagesEnd.value?.scrollIntoView({ behavior: "smooth", block: "end" });
  });

  // ── Send message ──────────────────────────────────────────────────────────

  const sendMessage = $(async () => {
    const word = input.value.trim();
    if (!word || loading.value) return;

    const baseId = crypto.randomUUID();
    messages.list.push({ id: `${baseId}-user`, role: "user", content: word });
    input.value = "";
    loading.value = true;

    const replyId = `${baseId}-reply`;

    try {
      const resp = await fetch("/api/explain", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ word, lens: activeLens.value, stream: true }),
      });

      if (!resp.ok) {
        let msg = `Server error ${resp.status}`;
        try {
          const err = await resp.json();
          msg = err.error ?? msg;
        } catch { /* ignore */ }
        messages.list.push({ id: replyId, role: "assistant", content: msg, lens: activeLens.value });
        return;
      }

      messages.list.push({ id: replyId, role: "assistant", content: "", lens: activeLens.value, streaming: true });
      messages.streamingId = replyId;
      loading.value = false;

      const reader = resp.body!.getReader();
      const decoder = new TextDecoder();
      let buffer = "";

      outer: while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        const parts = buffer.split("\n\n");
        buffer = parts.pop() ?? "";

        for (const part of parts) {
          let eventType = "message";
          let data = "";
          for (const line of part.split("\n")) {
            if (line.startsWith("event: ")) eventType = line.slice(7).trim();
            else if (line.startsWith("data: ")) data = line.slice(6);
          }
          if (eventType === "done") break outer;
          if (data) {
            const idx = messages.list.findIndex((m) => m.id === replyId);
            if (idx !== -1) messages.list[idx].content += data;
          }
        }
      }
    } catch {
      messages.list.push({
        id: replyId,
        role: "assistant",
        content: "Could not reach the WordLens backend. Is `cargo run` running on port 3001?",
        lens: activeLens.value,
      });
    } finally {
      const idx = messages.list.findIndex((m) => m.id === replyId);
      if (idx !== -1) messages.list[idx].streaming = false;
      messages.streamingId = null;
      loading.value = false;
    }
  });

  // ── Regenerate last response ──────────────────────────────────────────────

  const regenerate = $(async () => {
    const lastUser = [...messages.list].reverse().find((m) => m.role === "user");
    if (!lastUser) return;
    const lastUserIdx = messages.list.lastIndexOf(lastUser);
    messages.list.splice(lastUserIdx + 1);
    input.value = lastUser.content;
    await sendMessage();
  });

  // ── Helpers ───────────────────────────────────────────────────────────────

  const lensInfo = (id?: Lens) => LENSES.find((l) => l.id === id);
  const hasResponses = () => messages.list.some((m) => m.role === "assistant");

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class={["app flex flex-col h-dvh w-full", `lens-${activeLens.value}`]}>

      {/* ── Header ─────────────────────────────────────────────────────── */}
      <header class="flex items-center gap-3 px-5 py-3 border-b border-[var(--bot-border)] bg-[var(--bg-secondary)] shrink-0">
        <div class="w-9 h-9 rounded-xl bg-[var(--accent)] flex items-center justify-center text-white shrink-0 shadow-sm">
          <IconEye />
        </div>
        <div>
          <h1 class="text-[1rem] font-bold tracking-tight leading-none text-[var(--accent)]">
            WordLens
          </h1>
          <p class="text-[0.7rem] text-[var(--text-secondary)] mt-0.5 leading-none">
            Understand anything through multiple perspectives
          </p>
        </div>
      </header>

      {/* ── Lens selector ──────────────────────────────────────────────── */}
      <nav
        class="flex gap-1.5 px-4 py-2.5 bg-[var(--bg-secondary)] border-b border-[var(--bot-border)] overflow-x-auto scrollbar-hide shrink-0"
        aria-label="Select lens"
      >
        {LENSES.map((lens) => (
          <button
            key={lens.id}
            class={[
              "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold whitespace-nowrap shrink-0 border transition-all duration-200",
              activeLens.value === lens.id
                ? "bg-[var(--accent)] text-white border-transparent shadow-sm scale-[1.02]"
                : "text-[var(--text-secondary)] border-[var(--bot-border)] hover:bg-[var(--accent-light)] hover:text-[var(--text-primary)] hover:border-transparent hover:-translate-y-px",
            ]}
            onClick$={() => { activeLens.value = lens.id; }}
            aria-pressed={activeLens.value === lens.id}
            title={lens.tagline}
          >
            {(() => { const Icon = LENS_ICONS[lens.id]; return <Icon />; })()}
            <span class="hidden sm:inline">{lens.label}</span>
            <span class="hidden lg:inline opacity-60 font-normal">— {lens.tagline}</span>
          </button>
        ))}
      </nav>

      {/* ── Chat area ──────────────────────────────────────────────────── */}
      <main
        class="flex-1 overflow-y-auto px-4 py-5 flex flex-col gap-3 chat-scrollbar chat-inner"
        ref={chatArea}
      >

        {/* Empty state */}
        {messages.list.length === 0 && (
          <div class="flex-1 flex flex-col items-center justify-center gap-4 px-4 py-16 text-center">
            <div class="text-[var(--accent)] pulse-icon opacity-90">
              <IconSparkles />
            </div>
            <div class="space-y-1">
              <p class="font-semibold text-[var(--text-primary)] text-[0.95rem]">
                Enter any word, concept, or idea
              </p>
              <p class="text-xs text-[var(--text-muted)]">
                Pick a lens above to shape how it's explained
              </p>
            </div>
            <div class="flex flex-wrap gap-2 justify-center mt-1">
              {["entropy", "democracy", "recursion", "love", "gravity"].map((ex) => (
                <button
                  key={ex}
                  class="px-3 py-1.5 rounded-full bg-[var(--accent-light)] text-[var(--accent-bright)] text-xs font-medium border border-[var(--bot-border)] hover:bg-[var(--accent)] hover:text-white hover:border-transparent hover:-translate-y-px transition-all duration-200 italic"
                  onClick$={() => { input.value = ex; }}
                >
                  {ex}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Messages */}
        {messages.list.map((msg) => {
          const info = lensInfo(msg.lens);
          return (
            <div
              key={msg.id}
              class={[
                "flex flex-col max-w-[82%] msg-in",
                msg.role === "user" ? "self-end items-end" : "self-start items-start",
              ]}
            >
              {/* Lens badge on assistant messages */}
              {msg.role === "assistant" && info && (
                <div class="flex items-center gap-1 mb-1 px-1">
                  <span class="flex items-center gap-1 text-[0.65rem] font-bold uppercase tracking-widest text-[var(--badge-text)]">
                    {(() => { const Icon = LENS_ICONS[info.id]; return <Icon />; })()}
                    {info.label}
                  </span>
                </div>
              )}

              {/* Bubble */}
              <div
                class={[
                  "px-4 py-2.5 leading-relaxed text-[0.91rem] break-words whitespace-pre-wrap",
                  msg.role === "user"
                    ? "bg-[var(--user-bg)] text-[var(--user-text)] rounded-2xl rounded-br-sm shadow-sm"
                    : "bg-[var(--bot-bg)] text-[var(--bot-text)] border border-[var(--bot-border)] rounded-2xl rounded-tl-sm",
                ]}
              >
                {msg.content}
                {msg.streaming && <span class="cursor ml-0.5" aria-hidden="true">▌</span>}
              </div>
            </div>
          );
        })}

        {/* Typing indicator */}
        {loading.value && (
          <div class="self-start msg-in">
            <div class="typing-dots flex gap-1.5 px-4 py-3 bg-[var(--bot-bg)] border border-[var(--bot-border)] rounded-2xl rounded-tl-sm">
              <span />
              <span />
              <span />
            </div>
          </div>
        )}

        <div ref={messagesEnd} class="h-px shrink-0" />
      </main>

      {/* ── Input bar ──────────────────────────────────────────────────── */}
      <footer class="px-4 pb-5 pt-3 bg-[var(--bg-secondary)] border-t border-[var(--bot-border)] shrink-0 flex flex-col gap-2">

        {/* Input row */}
        <div class="flex items-center gap-2 bg-[var(--input-bg)] border border-[var(--input-border)] rounded-2xl pl-3 pr-1.5 py-1.5 transition-colors duration-200 input-ring">

          {/* Active lens chip */}
          {(() => { const li = lensInfo(activeLens.value); return li ? (
            <div class="flex items-center gap-1 text-[0.68rem] font-bold text-[var(--badge-text)] bg-[var(--badge-bg)] rounded-full px-2 py-0.5 shrink-0 select-none">
              {(() => { const Icon = LENS_ICONS[li.id]; return <Icon />; })()}
              <span class="hidden sm:inline">{li.label}</span>
            </div>
          ) : null; })()}

          <input
            class="flex-1 bg-transparent border-none outline-none text-[var(--text-primary)] text-[0.9rem] placeholder:text-[var(--text-muted)] min-w-0 py-0.5"
            type="text"
            placeholder="Enter a word or concept…"
            value={input.value}
            onInput$={(e) => { input.value = (e.target as HTMLInputElement).value; }}
            onKeyDown$={(e) => { if (e.key === "Enter" && !e.shiftKey) sendMessage(); }}
            disabled={loading.value}
            aria-label="Word or concept input"
          />

          {/* Send button */}
          <button
            class="w-9 h-9 rounded-xl bg-[var(--accent)] text-white flex items-center justify-center shrink-0 transition-all duration-150 hover:opacity-90 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed shadow-sm"
            onClick$={sendMessage}
            disabled={loading.value || !input.value.trim()}
            aria-label="Send"
          >
            {loading.value
              ? <span class="spinner" aria-hidden="true" />
              : <IconSend />
            }
          </button>
        </div>

        {/* Secondary actions */}
        {hasResponses() && (
          <div class="flex gap-2 justify-end">
            <button
              class="flex items-center gap-1.5 text-[0.72rem] font-medium text-[var(--text-secondary)] hover:text-[var(--accent-bright)] px-3 py-1.5 rounded-full border border-[var(--bot-border)] hover:bg-[var(--accent-light)] hover:border-transparent transition-all duration-200 disabled:opacity-40"
              onClick$={regenerate}
              disabled={loading.value}
            >
              <IconRefreshCw />
              Regenerate
            </button>
            <button
              class="flex items-center gap-1.5 text-[0.72rem] font-medium text-[var(--text-secondary)] hover:text-red-400 px-3 py-1.5 rounded-full border border-[var(--bot-border)] hover:bg-red-500/10 hover:border-red-500/20 transition-all duration-200"
              onClick$={() => { messages.list.splice(0, messages.list.length); }}
            >
              <IconTrash2 />
              Clear
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
      content: "Understand any word, concept, or idea through multiple AI-powered perspectives.",
    },
    { property: "og:title", content: "WordLens AI" },
    {
      property: "og:description",
      content: "Multi-perspective AI explanations powered by Llama 3.",
    },
  ],
};
