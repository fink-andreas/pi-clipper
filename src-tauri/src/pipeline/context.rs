use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDecision {
    pub is_terminal: bool,
    pub confidence: f32,
    pub process_name: Option<String>,
    pub window_title: Option<String>,
    pub matched_signature: Option<String>,
}

impl ContextDecision {
    pub fn unknown() -> Self {
        Self {
            is_terminal: false,
            confidence: 0.0,
            process_name: None,
            window_title: None,
            matched_signature: None,
        }
    }
}
