use anyhow::Result;
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClipboardWriteFingerprint {
    pub hash: String,
    pub timestamp: chrono::DateTime<Utc>,
}

pub struct ClipboardWriter {
    clipboard: Option<arboard::Clipboard>,
    recent_writes: VecDeque<ClipboardWriteFingerprint>,
    max_writes: usize,
    self_write_window_ms: i64,
}

impl ClipboardWriter {
    pub fn new(max_writes: usize, self_write_window_ms: i64) -> Result<Self> {
        Ok(Self {
            clipboard: arboard::Clipboard::new().ok(),
            recent_writes: VecDeque::new(),
            max_writes,
            self_write_window_ms,
        })
    }

    pub fn write_clipboard(&mut self, text: &str) -> Result<bool> {
        let hash = compute_hash(text);

        if self.is_self_write(&hash) {
            tracing::debug!("skipping self-write: hash {} matches recent write", hash);
            return Ok(false);
        }

        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(text.to_string())?;

            let fingerprint = ClipboardWriteFingerprint {
                hash: hash.clone(),
                timestamp: Utc::now(),
            };
            self.record_write(fingerprint);

            tracing::debug!("wrote cleaned clipboard text (hash: {})", hash);
            Ok(true)
        } else {
            Err(anyhow::anyhow!("clipboard unavailable"))
        }
    }

    fn is_self_write(&self, hash: &str) -> bool {
        let now = Utc::now();
        self.recent_writes
            .iter()
            .any(|fp| fp.hash == hash && {
                let delta = now - fp.timestamp;
                delta.num_milliseconds().abs() < self.self_write_window_ms
            })
    }

    fn record_write(&mut self, fingerprint: ClipboardWriteFingerprint) {
        self.recent_writes.push_back(fingerprint);
        while self.recent_writes.len() > self.max_writes {
            self.recent_writes.pop_front();
        }
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
    fn test_hash_consistency() {
        let hash1 = compute_hash("test");
        let hash2 = compute_hash("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_self_write_detection() {
        let mut writer = ClipboardWriter::new(10, 500).unwrap();
        let hash = compute_hash("test");

        // First write should not be considered self-write
        assert!(!writer.is_self_write(&hash));

        // Record the write
        let now = Utc::now();
        writer.record_write(ClipboardWriteFingerprint {
            hash: hash.clone(),
            timestamp: now,
        });

        // Immediate second write should be flagged as self-write
        assert!(writer.is_self_write(&hash));

        // Different hash should not be flagged
        let other_hash = compute_hash("other");
        assert!(!writer.is_self_write(&other_hash));
    }

    #[test]
    fn test_write_fingerprint_eviction() {
        let mut writer = ClipboardWriter::new(2, 500).unwrap();
        let now = Utc::now();

        writer.record_write(ClipboardWriteFingerprint {
            hash: "1".to_string(),
            timestamp: now,
        });
        writer.record_write(ClipboardWriteFingerprint {
            hash: "2".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
        });
        writer.record_write(ClipboardWriteFingerprint {
            hash: "3".to_string(),
            timestamp: now + chrono::Duration::seconds(2),
        });

        // First hash should be evicted
        assert!(!writer.is_self_write("1"));
        assert!(writer.is_self_write("2"));
        assert!(writer.is_self_write("3"));
    }
}