use serde::Deserialize;

use super::Config;
use crate::error::AppResult;

/// Mechanical settings from `bridge.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub node: String,
    pub priority_limit: usize,
    pub claude_command: String,
    pub activity_ttl_days: i64,
    pub run_ttl_days: i64,
}

pub fn load(config: &Config) -> AppResult<Settings> {
    let text = std::fs::read_to_string(config.root().join("bridge.toml"))?;
    Ok(toml::from_str(&text)?)
}
