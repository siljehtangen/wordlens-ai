use chrono::Utc;
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::VecDeque;

use crate::types::Lens;

use crate::MAX_HISTORY_ENTRIES;

#[derive(Clone, Serialize)]
pub struct HistoryEntry {
    pub word: String,
    pub lens: Lens,
    pub snippet: String,
    pub timestamp: i64,
}

pub struct History(RwLock<VecDeque<HistoryEntry>>);

impl Default for History {
    fn default() -> Self {
        Self(RwLock::new(VecDeque::with_capacity(MAX_HISTORY_ENTRIES)))
    }
}

impl History {
    pub fn push(&self, word: String, lens: Lens, explanation: &str) {
        let snippet = explanation.chars().take(120).collect::<String>();
        let entry = HistoryEntry {
            word,
            lens,
            snippet,
            timestamp: Utc::now().timestamp(),
        };
        let mut q = self.0.write();
        if q.len() == MAX_HISTORY_ENTRIES {
            q.pop_front();
        }
        q.push_back(entry);
    }

    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let q = self.0.read();
        q.iter().rev().take(limit).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_returns_newest_first() {
        let h = History::default();
        h.push("alpha".to_string(), Lens::Simple, "explanation alpha");
        h.push("beta".to_string(), Lens::Simple, "explanation beta");
        let recent = h.recent(10);
        assert_eq!(recent[0].word, "beta");
        assert_eq!(recent[1].word, "alpha");
    }

    #[test]
    fn history_caps_at_max_entries() {
        let h = History::default();
        for i in 0..=MAX_HISTORY_ENTRIES {
            h.push(format!("word{i}"), Lens::Simple, "exp");
        }
        assert_eq!(h.recent(100).len(), MAX_HISTORY_ENTRIES);
    }

    #[test]
    fn snippet_truncated_at_120_chars() {
        let h = History::default();
        let long = "x".repeat(200);
        h.push("word".to_string(), Lens::Simple, &long);
        let recent = h.recent(1);
        assert_eq!(recent[0].snippet.chars().count(), 120);
    }

    #[test]
    fn recent_limit_respected() {
        let h = History::default();
        for i in 0..10 {
            h.push(format!("w{i}"), Lens::Simple, "exp");
        }
        assert_eq!(h.recent(3).len(), 3);
    }

    #[test]
    fn empty_history_returns_empty_vec() {
        let h = History::default();
        assert!(h.recent(10).is_empty());
    }
}
