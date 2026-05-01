use crate::prompts;
use crate::types::{ExplainRequest, Lens};

pub const MAX_WORD_LEN: usize = 200;
pub const OLLAMA_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

pub fn validate_request(payload: &ExplainRequest) -> Result<(), String> {
    let word = payload.word.trim();
    if word.is_empty() {
        return Err("Word cannot be empty.".to_string());
    }
    if word.len() > MAX_WORD_LEN {
        return Err(format!("Word is too long (max {MAX_WORD_LEN} characters)."));
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

pub fn ollama_body(model: &str, prompt: &str, stream: bool, num_predict: u32) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": stream,
        "options": {
            "num_predict": num_predict,
            "num_ctx": 2048,
            "temperature": 0.7,
            "top_p": 0.9,
            "repeat_penalty": 1.1
        }
    })
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

    #[test]
    fn lens_display_is_lowercase() {
        assert_eq!(Lens::Simple.to_string(), "simple");
        assert_eq!(Lens::Cyberpunk.to_string(), "cyberpunk");
    }
}
