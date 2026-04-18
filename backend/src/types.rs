use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub word: String,
    pub lens: String,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}
fn default_limit() -> usize {
    20
}

#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub lens: String,
    pub word: String,
    pub cached: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Typed shape of a single Ollama streaming chunk.
#[derive(Deserialize)]
pub struct OllamaChunk {
    #[serde(default)]
    pub response: String,
    #[serde(default)]
    pub done: bool,
}
