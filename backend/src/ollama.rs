use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::prompts;
use crate::types::{ExplainRequest, Lens};

pub const MAX_WORD_LEN: usize = 200;
pub const OLLAMA_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

const NUM_CTX: u32 = 2048;
const TEMPERATURE: f64 = 0.7;
const TOP_P: f64 = 0.9;
const REPEAT_PENALTY: f64 = 1.1;

pub fn validate_request(payload: &ExplainRequest) -> Result<(), AppError> {
    let word = payload.word.trim();
    if word.is_empty() {
        return Err(AppError::InvalidRequest("Word cannot be empty.".to_string()));
    }
    if word.chars().count() > MAX_WORD_LEN {
        return Err(AppError::InvalidRequest(format!(
            "Word is too long (max {MAX_WORD_LEN} characters)."
        )));
    }
    Ok(())
}

pub fn build_prompt(word: &str, lens: Lens) -> String {
    let word = word.trim();
    let template = match lens {
        Lens::Simple    => prompts::PROMPT_SIMPLE,
        Lens::Learning  => prompts::PROMPT_LEARNING,
        Lens::Game      => prompts::PROMPT_GAME,
        Lens::Cyberpunk => prompts::PROMPT_CYBERPUNK,
        Lens::Poetic    => prompts::PROMPT_POETIC,
    };
    template.replace("{word}", word)
}

pub fn lens_token_limit(lens: Lens) -> u32 {
    match lens {
        Lens::Simple    => 180,
        Lens::Learning  => 520,
        Lens::Game      => 420,
        Lens::Cyberpunk => 320,
        Lens::Poetic    => 360,
    }
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: u32,
    num_ctx: u32,
    temperature: f64,
    top_p: f64,
    repeat_penalty: f64,
}

#[derive(Serialize)]
pub struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

/// Typed shape of a single Ollama streaming/non-streaming response chunk.
#[derive(Deserialize)]
pub struct OllamaChunk {
    #[serde(default)]
    pub response: String,
    #[serde(default)]
    pub done: bool,
}

pub fn ollama_body(model: &str, prompt: &str, stream: bool, num_predict: u32) -> OllamaRequest {
    OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream,
        options: OllamaOptions {
            num_predict,
            num_ctx: NUM_CTX,
            temperature: TEMPERATURE,
            top_p: TOP_P,
            repeat_penalty: REPEAT_PENALTY,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(word: &str, lens: Lens) -> ExplainRequest {
        ExplainRequest { word: word.to_string(), lens, stream: false }
    }

    #[test]
    fn empty_word_rejected() {
        assert!(validate_request(&req("", Lens::Simple)).is_err());
    }

    #[test]
    fn whitespace_word_rejected() {
        assert!(validate_request(&req("   ", Lens::Simple)).is_err());
    }

    #[test]
    fn word_at_limit_passes() {
        let w = "a".repeat(MAX_WORD_LEN);
        assert!(validate_request(&req(&w, Lens::Simple)).is_ok());
    }

    #[test]
    fn word_over_limit_rejected() {
        let w = "a".repeat(MAX_WORD_LEN + 1);
        assert!(validate_request(&req(&w, Lens::Simple)).is_err());
    }

    #[test]
    fn multibyte_chars_count_as_one_each() {
        // "é" is 2 bytes but 1 char — MAX_WORD_LEN é's must pass
        let w = "é".repeat(MAX_WORD_LEN);
        assert!(validate_request(&req(&w, Lens::Simple)).is_ok());
        let w = "é".repeat(MAX_WORD_LEN + 1);
        assert!(validate_request(&req(&w, Lens::Simple)).is_err());
    }

    #[test]
    fn build_prompt_substitutes_word_for_all_lenses() {
        for lens in [Lens::Simple, Lens::Learning, Lens::Game, Lens::Cyberpunk, Lens::Poetic] {
            let p = build_prompt("quantum", lens);
            assert!(p.contains("quantum"), "{lens:?}: word not in prompt");
            assert!(!p.contains("{word}"), "{lens:?}: placeholder not replaced");
        }
    }

    #[test]
    fn token_limits_all_positive() {
        for lens in [Lens::Simple, Lens::Learning, Lens::Game, Lens::Cyberpunk, Lens::Poetic] {
            assert!(lens_token_limit(lens) > 0, "{lens:?}: token limit is zero");
        }
    }

}
