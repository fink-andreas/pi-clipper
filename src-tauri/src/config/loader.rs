use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config::schema::AppConfig;

pub fn load_config(config_path: &Path) -> Result<AppConfig> {
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(config_path)?;
    let cfg = serde_json::from_str::<AppConfig>(&content).unwrap_or_default();
    Ok(cfg)
}
