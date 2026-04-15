use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub dedupe_window: usize,
    pub terminal_confidence_threshold: f32,
    pub event_logging_enabled: bool,
    pub log_retention_days: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dedupe_window: 20,
            terminal_confidence_threshold: 0.7,
            event_logging_enabled: false,
            log_retention_days: 7,
        }
    }
}
