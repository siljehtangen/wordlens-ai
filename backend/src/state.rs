use moka::future::Cache;
use std::sync::Arc;

use crate::history::History;

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub ollama_generate_url: String,
    pub model: String,
    pub cache: Cache<(String, String), String>,
    pub history: Arc<History>,
}
