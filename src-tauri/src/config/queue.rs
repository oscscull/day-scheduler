//! The `[queue]` section of `quarterdeck.toml`: items Claude works on one at a time, in order.

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct QueueSettings {
    /// Item ids, first to work on first.
    pub items: Vec<String>,
}
