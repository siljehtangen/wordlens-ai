use chrono::Utc;
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::VecDeque;

const MAX_ENTRIES: usize = 50;

#[derive(Clone, Serialize)]
pub struct HistoryEntry {
    pub word: String,
    pub lens: String,
    pub snippet: String, // first 120 chars of explanation
    pub timestamp: i64,  // Unix seconds (UTC)
}

pub struct History(RwLock<VecDeque<HistoryEntry>>);

impl Default for History {
    fn default() -> Self {
        Self(RwLock::new(VecDeque::with_capacity(MAX_ENTRIES)))
    }
}

impl History {
    pub fn push(&self, word: String, lens: String, explanation: &str) {
        let snippet = explanation.chars().take(120).collect::<String>();
        let entry = HistoryEntry {
            word,
            lens,
            snippet,
            timestamp: Utc::now().timestamp(),
        };
        let mut q = self.0.write();
        if q.len() == MAX_ENTRIES {
            q.pop_front();
        }
        q.push_back(entry);
    }

    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let q = self.0.read();
        q.iter().rev().take(limit).cloned().collect()
    }
}
