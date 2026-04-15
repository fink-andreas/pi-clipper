use crate::pipeline::watcher::ClipboardChanged;

pub struct ClipboardObserver {
    poll_interval_ms: u64,
    last_hash: Option<String>,
}

impl ClipboardObserver {
    pub fn new(poll_interval_ms: u64) -> Self {
        Self {
            poll_interval_ms,
            last_hash: None,
        }
    }

    pub async fn next_change(&mut self) -> Option<ClipboardChanged> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.poll_interval_ms)).await;

        let text = read_clipboard()?;
        let hash = compute_hash(&text);

        if Some(&hash) == self.last_hash.as_ref() {
            return None;
        }

        self.last_hash = Some(hash.clone());

        Some(ClipboardChanged {
            timestamp: chrono::Utc::now(),
            text,
            hash,
        })
    }
}

fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

fn compute_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}