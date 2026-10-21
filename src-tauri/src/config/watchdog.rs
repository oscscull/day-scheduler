//! The `[watchdog]` section of `quarterdeck.toml`: keeping Claude moving without nagging it.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WatchdogSettings {
    pub enabled: bool,
    /// How long an item Claude holds can go without an update before Meatpuppet checks in.
    pub quiet_minutes: i64,
    /// Check-ins per quiet spell before Meatpuppet stops and tells the user instead.
    pub max_nudges: u32,
}

impl Default for WatchdogSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            quiet_minutes: 45,
            max_nudges: 2,
        }
    }
}
