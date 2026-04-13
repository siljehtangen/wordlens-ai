# WordLens AI

**Understand any word, concept, or idea through multiple AI-powered lenses.**

WordLens AI is a multi-perspective language tool built entirely in Rust — a Leptos/WASM frontend and an Axum backend — with Llama 3 running locally via Ollama. Instead of a single static definition, it explains the same concept in five completely different ways, each with its own visual identity.

---

## What it does

Type a word like *entropy*, *democracy*, or *love* and WordLens returns an explanation shaped by whichever lens you've selected. Switch lenses instantly to see the same concept reframed — the UI shifts its entire colour theme to match.

---

## Lenses

| Lens | Theme | Style |
|------|-------|-------|
| **Simple** | Soft blue | Clear, friendly, no jargon |
| **Learning** | Deep purple | Structured, educational, with examples |
| **Game** | Neon green | Reframed as a game mechanic or system |
| **Cyberpunk** | Dark + neon pink/cyan | Tech-noir, futuristic, atmospheric |
| **Poetic** | Warm amber/gold | Metaphorical, imagery-driven prose poetry |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | [Leptos](https://leptos.dev) 0.7 (Rust → WASM, CSR) |
| Build tool | [Trunk](https://trunkrs.dev) |
| Styling | Tailwind CSS (CDN) + CSS custom properties |
| Backend | [Axum](https://github.com/tokio-rs/axum) (Rust) |
| Cache | [Moka](https://github.com/moka-rs/moka) (async in-memory, 1 h TTL) |
| AI Runtime | [Ollama](https://ollama.com) |
| Model | Llama 3 (`llama3`) |

The entire stack — from the pixel in the browser to the HTTP call to Ollama — is written in Rust.

---

## Architecture

```
User (browser)
     │
     ▼
Leptos WASM app     (:8080 dev via Trunk)
     │  POST /api/explain  { word, lens, stream }
     ▼
Axum REST API       (:3001)
     │  checks Moka cache → cache hit returns immediately
     │  POST /api/generate  { model, prompt, stream }
     ▼
Ollama              (:11434)
     │
     ▼
token stream → SSE → Leptos reactive UI → chat bubble
```

---

## Getting Started

### Prerequisites

| Tool | Install |
|------|---------|
| Rust + Cargo | https://rustup.rs |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| Trunk | `cargo install trunk` |
| Ollama | https://ollama.com |

### 1. Pull the model

```bash
ollama pull llama3
```

### 2. Start Ollama

```bash
ollama serve
```

### 3. Start the backend

```bash
cd backend
cargo run --release
```

Listens on **http://localhost:3001**. Set `BIND_ADDR`, `OLLAMA_URL`, or `OLLAMA_MODEL` environment variables to override defaults.

### 4. Start the frontend

```bash
cd frontend
trunk serve
```

Open **http://localhost:8080**. Trunk proxies `/api/*` to the backend automatically.

---

## Production build

```bash
# Build the WASM frontend
cd frontend && trunk build --release

# Run the backend (serves frontend/dist/ as static files)
cd ../backend && cargo run --release
```

The backend serves the compiled frontend from `../frontend/dist` by default. Override with `FRONTEND_DIST`.

---

## Project Structure

```
wordlens-ai/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs       # Axum server, Moka cache, /api/explain, /api/history
│       ├── history.rs    # In-memory ring buffer (last 50 explanations)
│       └── prompts.rs    # Prompt templates for each lens
│
├── frontend/
│   ├── Cargo.toml        # Leptos + wasm-bindgen + wasm-streams
│   ├── Trunk.toml        # Build config + dev proxy
│   ├── index.html        # Entry point — Tailwind CDN, CSS variables, keyframes
│   └── src/
│       └── main.rs       # Full Leptos app: components, SSE streaming, state
│
└── README.md
```

---

## API

### `POST /api/explain`

**Request:**
```json
{
  "word": "entropy",
  "lens": "cyberpunk",
  "stream": false
}
```
`lens` must be one of: `simple` | `learning` | `game` | `cyberpunk` | `poetic`

**Response (`stream: false`):**
```json
{
  "explanation": "In the sprawling data-hive of New Shanghai...",
  "lens": "cyberpunk",
  "word": "entropy",
  "cached": false
}
```
Non-streaming responses are cached (word + lens key). `cached: true` means the response was served from memory without hitting Ollama.

**Response (`stream: true`):**
Server-Sent Events. Each event carries one token as `data`. A final `event: done` signals completion. Streaming responses are recorded in history but not cached.

---

### `GET /api/history?limit=20`

Returns the last N explanations (max 50), most recent first.

```json
[
  {
    "word": "entropy",
    "lens": "cyberpunk",
    "snippet": "In the sprawling data-hive of New Shanghai...",
    "timestamp": 1744000000
  }
]
```

---

### `GET /health`

```json
{ "status": "ok" }
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BIND_ADDR` | `0.0.0.0:3001` | Backend listen address |
| `OLLAMA_URL` | `http://127.0.0.1:11434` | Ollama base URL |
| `OLLAMA_MODEL` | `llama3` | Model name passed to Ollama |
| `FRONTEND_DIST` | `../frontend/dist` | Path to compiled frontend assets |

---

## Development Notes

- Switching lenses mid-flight is safe — the in-flight request completes with its original lens badge.
- The Moka cache uses a `(word, lens)` key (word is lowercased and trimmed). Cache entries expire after 1 hour.
- History is in-memory only — it resets on backend restart.
- To use a different model, set `OLLAMA_MODEL=llama3.2` (or any model you have pulled).
