//! The `[alerts]` section of `bridge.toml`: the loud, on-top countdown before meetings.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlertSettings {
    pub enabled: bool,
    /// How long before a meeting the alert opens.
    pub lead_minutes: i64,
    /// How long after the start it stays up if nobody acts.
    pub linger_minutes: i64,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            lead_minutes: 5,
            linger_minutes: 3,
        }
    }
}
