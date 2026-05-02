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
    crate::DEFAULT_HISTORY_LIMIT
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_display_is_lowercase() {
        assert_eq!(Lens::Simple.to_string(), "simple");
        assert_eq!(Lens::Cyberpunk.to_string(), "cyberpunk");
    }
}

