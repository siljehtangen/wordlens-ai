use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lens {
    Simple,
    Learning,
    Game,
    Cyberpunk,
    Poetic,
}

impl Lens {
    pub fn as_str(self) -> &'static str {
        match self {
            Lens::Simple    => "simple",
            Lens::Learning  => "learning",
            Lens::Game      => "game",
            Lens::Cyberpunk => "cyberpunk",
            Lens::Poetic    => "poetic",
        }
    }
}

impl std::fmt::Display for Lens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}
fn default_limit() -> usize {
    20
}

#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub word: String,
    pub lens: Lens,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub lens: Lens,
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
