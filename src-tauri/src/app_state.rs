use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

#[derive(Debug)]
pub struct AppStateInner {
    pub monitoring_enabled: bool,
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = base.join("pi-clipper");
        let log_dir = config_dir.join("logs");

        std::fs::create_dir_all(&config_dir)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(AppStateInner {
                monitoring_enabled: true,
                config_dir,
                log_dir,
            })),
        })
    }

    pub fn set_monitoring_enabled(&self, enabled: bool) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.monitoring_enabled = enabled;
        }
    }

    pub fn monitoring_enabled(&self) -> bool {
        self.inner
            .lock()
            .map(|s| s.monitoring_enabled)
            .unwrap_or(false)
    }

    pub fn log_dir(&self) -> Option<PathBuf> {
        self.inner.lock().ok().map(|s| s.log_dir.clone())
    }

    pub fn config_dir(&self) -> Option<PathBuf> {
        self.inner.lock().ok().map(|s| s.config_dir.clone())
    }
}
