use std::collections::VecDeque;

#[derive(Debug)]
pub struct DedupeGuard {
    recent: VecDeque<String>,
    max_entries: usize,
}

impl DedupeGuard {
    pub fn new(max_entries: usize) -> Self {
        Self {
            recent: VecDeque::new(),
            max_entries,
        }
    }

    pub fn seen_recently(&mut self, hash: &str) -> bool {
        if self.recent.iter().any(|h| h == hash) {
            return true;
        }

        self.recent.push_back(hash.to_string());
        while self.recent.len() > self.max_entries {
            self.recent.pop_front();
        }
        false
    }
}
