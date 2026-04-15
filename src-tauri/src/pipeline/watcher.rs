use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardChanged {
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub hash: String,
}

pub fn compute_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dedupe::DedupeGuard;

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
