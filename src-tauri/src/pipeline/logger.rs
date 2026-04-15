use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub context: Option<ContextLog>,
    pub input_hash: Option<String>,
    pub output_hash: Option<String>,
    pub input_preview: Option<String>,
    pub output_preview: Option<String>,
    pub changed: bool,
    pub actions: Vec<String>,
    pub duration_ms: u64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLog {
    pub is_terminal: bool,
    pub confidence: f32,
    pub process_name: Option<String>,
    pub window_title: Option<String>,
}

pub struct EventLogger {
    log_dir: PathBuf,
    log_file: PathBuf,
    retention_days: u32,
    max_preview: usize,
}

impl EventLogger {
    pub fn new(log_dir: PathBuf, retention_days: u32) -> Result<Self> {
        fs::create_dir_all(&log_dir)?;

        let log_file = log_dir.join("events.jsonl");

        Ok(Self {
            log_dir,
            log_file,
            retention_days,
            max_preview: 200,
        })
    }

    pub fn log(&mut self, record: &EventLog) -> Result<()> {
        self.append_jsonl(record)?;
        self.rotate_logs()?;
        Ok(())
    }

    fn append_jsonl(&self, record: &EventLog) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{line}")?;

        Ok(())
    }

    fn rotate_logs(&self) -> Result<()> {
        let now = SystemTime::now();
        let retention_duration = std::time::Duration::from_secs(self.retention_days as u64 * 86400);

        let entries = fs::read_dir(&self.log_dir)?;

        for entry in entries.flatten() {
            if let Ok(modified) = entry.metadata()?.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > retention_duration {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            fs::remove_file(path)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn truncate_preview(text: &str) -> String {
        if text.len() > 200 {
            let mut preview = text.chars().take(200).collect::<String>();
            preview.push_str("...");
            preview
        } else {
            text.to_string()
        }
    }

    pub fn open_logs_folder(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer").arg(&self.log_dir).spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(&self.log_dir).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(&self.log_dir).spawn()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_preview_short() {
        let text = "hello world";
        let preview = EventLogger::truncate_preview(text);
        assert_eq!(preview, "hello world");
    }

    #[test]
    fn test_truncate_preview_long() {
        let text = "a".repeat(300);
        let preview = EventLogger::truncate_preview(&text);
        assert!(preview.len() <= 204);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_serialization() {
        let log = EventLog {
            timestamp: Utc::now(),
            event_id: Uuid::new_v4().to_string(),
            context: None,
            input_hash: Some("abc123".to_string()),
            output_hash: Some("def456".to_string()),
            input_preview: Some("hello".to_string()),
            output_preview: Some("world".to_string()),
            changed: true,
            actions: vec!["strip_ansi".to_string(), "trim_blank_edges".to_string()],
            duration_ms: 12,
            status: "ok".to_string(),
            error: None,
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("event_id"));
    }
}