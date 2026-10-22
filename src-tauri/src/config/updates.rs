//! The `[updates]` section of `quarterdeck.toml`: installing new releases by itself.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UpdateSettings {
    pub enabled: bool,
    /// How often to look for a new release.
    pub check_minutes: i64,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            check_minutes: 30,
        }
    }
}
