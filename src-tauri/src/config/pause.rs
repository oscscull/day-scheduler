//! Pausing every schedule at once, stored as `pause.toml` so an agent can do it too.
//! No file means not paused. Resuming deletes the file.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Config;
use crate::error::AppResult;

/// An active pause. `until: None` means until the user resumes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pause {
    pub until: Option<DateTime<Utc>>,
}

fn path(config: &Config) -> PathBuf {
    config.root().join("pause.toml")
}

/// The current pause, if any. A pause whose end has passed is cleared.
pub fn current(config: &Config, now: DateTime<Utc>) -> AppResult<Option<Pause>> {
    let file = path(config);
    if !file.exists() {
        return Ok(None);
    }
    let pause: Pause = toml::from_str(&std::fs::read_to_string(&file)?)?;
    if pause.until.is_some_and(|until| until <= now) {
        std::fs::remove_file(file)?;
        return Ok(None);
    }
    Ok(Some(pause))
}

pub fn pause(config: &Config, until: Option<DateTime<Utc>>) -> AppResult<()> {
    let header = "# Written by Meatpuppet. Delete this file to resume all schedules.\n";
    let body = toml::to_string(&Pause { until })?;
    std::fs::write(path(config), format!("{header}{body}"))?;
    Ok(())
}

pub fn resume(config: &Config) -> AppResult<()> {
    let file = path(config);
    if file.exists() {
        std::fs::remove_file(file)?;
    }
    Ok(())
}
