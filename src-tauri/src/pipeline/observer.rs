use crate::pipeline::dedupe::DedupeGuard;
use crate::pipeline::watcher::{compute_hash, ClipboardChanged};

pub struct ClipboardObserver {
    poll_interval_ms: u64,
    dedupe: DedupeGuard,
}

impl ClipboardObserver {
    pub fn new(poll_interval_ms: u64, dedupe_window: usize) -> Self {
        Self {
            poll_interval_ms,
            dedupe: DedupeGuard::new(dedupe_window),
        }
    }

    pub async fn next_change(&mut self) -> Option<ClipboardChanged> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.poll_interval_ms)).await;

        let text = read_clipboard()?;
        let hash = compute_hash(&text);

        if self.dedupe.seen_recently(&hash) {
            return None;
        }

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
