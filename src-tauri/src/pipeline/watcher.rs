use anyhow::Result;
use arboard::Clipboard;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::pipeline::dedupe::DedupeGuard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardChanged {
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub hash: String,
}

pub struct SystemClipboardWatcher {
    clipboard: Option<Clipboard>,
    dedupe: DedupeGuard,
    last_text: Option<String>,
    poll_interval_ms: u64,
}

impl SystemClipboardWatcher {
    pub fn new(poll_interval_ms: u64) -> Result<Self> {
        Ok(Self {
            clipboard: Clipboard::new().ok(),
            dedupe: DedupeGuard::new(20),
            last_text: None,
            poll_interval_ms,
        })
    }

    pub async fn run(&mut self) -> Option<ClipboardChanged> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.poll_interval_ms)).await;

        let text = self.read_text()?;
        let hash = compute_hash(&text);

        if self.dedupe.seen_recently(&hash) {
            return None;
        }

        Some(ClipboardChanged {
            timestamp: Utc::now(),
            text,
            hash,
        })
    }

    fn read_text(&mut self) -> Option<String> {
        let clipboard = self.clipboard.as_mut()?;
        clipboard.get_text().ok()
    }
}

fn compute_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("hello");
        let hash2 = compute_hash("hello");
        let hash3 = compute_hash("world");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_dedupe_guard_rejects_duplicates() {
        let mut guard = DedupeGuard::new(5);
        let hash = "abc123";
        assert!(!guard.seen_recently(hash));
        assert!(guard.seen_recently(hash));
        assert!(guard.seen_recently(hash));
    }

    #[test]
    fn test_dedupe_guard_evicts_old() {
        let mut guard = DedupeGuard::new(3);
        guard.seen_recently("1");
        guard.seen_recently("2");
        guard.seen_recently("3");
        guard.seen_recently("4"); // evicts "1"
        assert!(!guard.seen_recently("1")); // should miss
        assert!(guard.seen_recently("2")); // still present
    }
}