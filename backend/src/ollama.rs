use crate::prompts;
use crate::types::ExplainRequest;

pub const MAX_WORD_LEN: usize = 200;
pub const VALID_LENSES: &[&str] = &["simple", "learning", "game", "cyberpunk", "poetic"];
pub const OLLAMA_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

pub fn validate_request(payload: &ExplainRequest) -> Result<(), String> {
    let word = payload.word.trim();
    if word.is_empty() {
        return Err("Word cannot be empty.".to_string());
    }
    if word.len() > MAX_WORD_LEN {
        return Err(format!("Word is too long (max {MAX_WORD_LEN} characters)."));
    }
    if !VALID_LENSES.contains(&payload.lens.as_str()) {
        return Err(format!(
            "Unknown lens '{}'. Valid options: {}.",
            payload.lens,
            VALID_LENSES.join(", ")
        ));
    }
    Ok(())
}

pub fn build_prompt(word: &str, lens: &str) -> String {
    let word = word.trim();
    let template = match lens {
        "simple"    => prompts::PROMPT_SIMPLE,
        "learning"  => prompts::PROMPT_LEARNING,
        "game"      => prompts::PROMPT_GAME,
        "cyberpunk" => prompts::PROMPT_CYBERPUNK,
        "poetic"    => prompts::PROMPT_POETIC,
        _           => "Explain '{word}' clearly and concisely.",
    };
    template.replace("{word}", word)
}

pub fn lens_token_limit(lens: &str) -> u32 {
    match lens {
        "simple"    => 180,
        "learning"  => 520,
        "game"      => 420,
        "cyberpunk" => 320,
        "poetic"    => 360,
        _           => 320,
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
