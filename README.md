# 🔍 WordLens AI

**Understand any word, concept, or idea through multiple AI-powered lenses.**

WordLens AI is a multi-perspective language tool built with a Rust/Axum backend, Qwik frontend, and Llama 3 running locally via Ollama. Instead of giving you a single static definition, it explains the same concept in five completely different ways — each tailored to a different way of thinking.

---

## 🧠 What it does

Type a word like *entropy*, *democracy*, or *love* and WordLens returns an explanation shaped by whichever lens you've selected. Switch lenses instantly to see the same concept reframed — the UI adapts its entire colour identity to match.

---

## 🔍 Lenses

| Lens | Theme | Style |
|------|-------|-------|
| 📚 **Simple** | Soft blue | Clear, friendly, no jargon |
| 🧠 **Learning** | Deep purple | Structured, educational, with examples |
| 🎮 **Game** | Neon green | Reframed as a game mechanic or system |
| 🏙️ **Cyberpunk** | Dark + neon pink/cyan | Tech-noir, futuristic, atmospheric |
| 📖 **Poetic** | Warm amber/gold | Metaphorical, imagery-driven prose poetry |

---

## 🎨 Dynamic Colour System

The interface shifts its entire colour theme when you switch lens — background, chat bubbles, badges, inputs, and hover states all change. Every mode has its own visual identity that reinforces the tone of the explanation.

---

## ⚙️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | [Qwik](https://qwik.dev) + Qwik City (SSR) |
| Backend | [Axum](https://github.com/tokio-rs/axum) (Rust) |
| AI Runtime | [Ollama](https://ollama.com) |
| Model | Llama 3 (`llama3`) |

---

## 🏗️ Architecture

```
User (browser)
     │
     ▼
Qwik City frontend  (:5173 dev / :4173 preview)
     │  POST /api/explain  { word, lens }
     ▼
Axum REST API        (:8080)
     │  POST /api/generate  { model, prompt, stream }
     ▼
Ollama               (:11434)
     │
     ▼
Llama 3 response → styled chat bubble
```

---

## 🚀 Getting Started

### Prerequisites

| Tool | Install |
|------|---------|
| Rust + Cargo | https://rustup.rs |
| Node.js ≥ 18 | https://nodejs.org |
| Ollama | https://ollama.com |

### 1. Pull the model

```bash
ollama pull llama3
```

### 2. Start Ollama

```bash
ollama serve
```

Ollama listens on `http://localhost:11434` by default.

### 3. Start the backend

```bash
cd backend
cargo run --release
```

The server starts on **http://localhost:8080**.

> First build will take a minute while Cargo fetches dependencies.

### 4. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

Open **http://localhost:5173** in your browser.

---

## 📁 Project Structure

```
wordlens-ai/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs          # Axum server, prompt builder, Ollama client
│
├── frontend/
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── public/
│   │   ├── favicon.svg
│   │   └── manifest.json
│   └── src/
│       ├── global.css        # All theming & layout styles
│       ├── root.tsx          # Qwik City app shell
│       ├── entry.ssr.tsx     # SSR entry point
│       └── routes/
│           ├── layout.tsx    # Root layout (pass-through)
│           └── index.tsx     # Main page — all chat UI & state
│
└── README.md
```

---

## 🔌 API

### `POST /api/explain`

**Request body:**

```jsonc
{
  "word": "entropy",      // required — the word or concept to explain
  "lens": "cyberpunk",    // required — one of: simple | learning | game | cyberpunk | poetic
  "stream": false         // optional — set true for SSE token streaming
}
```

**Response (stream: false):**

```json
{
  "explanation": "In the sprawling data-hive of New Shanghai...",
  "lens": "cyberpunk",
  "word": "entropy"
}
```

**Response (stream: true):**  
Server-Sent Events. Each event carries one token as `data`. A final `event: done` signals completion.

---

## 🛠️ Development Notes

- The Vite dev server proxies `/api/*` to `http://localhost:8080`, so the frontend and backend can run independently.
- Switching lenses while a response is in flight is safe — the in-flight request completes with its original lens badge.
- The `stream: false` mode in the frontend is the default. To enable token-by-token streaming, set `stream: true` in the `fetch` call in [frontend/src/routes/index.tsx](frontend/src/routes/index.tsx) — the backend already supports it.
- To use a different model (e.g. `llama3.2` or `mistral`), change the `"model"` field in `backend/src/main.rs` → `build_prompt`'s caller in `explain_json` / `explain_stream`.

---

## 🌍 Vision

> From static definitions → to dynamic, multi-perspective thinking.

WordLens AI turns learning into something visual, intuitive, and playful by combining local AI, reactive UI, and expressive design.
