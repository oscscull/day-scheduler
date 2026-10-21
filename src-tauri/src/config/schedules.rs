//! Schedules stored as `schedules/<id>.md`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Config, frontmatter};
use crate::domain::schedule::{Autonomy, Schedule, Scope, Trigger};
use crate::error::{AppError, AppResult};

/// The TOML part of a schedule file.
#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    name: String,
    node: String,
    scope: Scope,
    autonomy: Autonomy,
    enabled: bool,
    #[serde(default)]
    request: String,
    trigger: Trigger,
}

/// Schedules that loaded, plus a message for each file that did not.
#[derive(Debug, Default)]
pub struct Loaded {
    pub schedules: Vec<Schedule>,
    pub problems: Vec<String>,
}

fn dir(config: &Config) -> PathBuf {
    config.root().join("schedules")
}

fn path(config: &Config, id: &str) -> PathBuf {
    dir(config).join(format!("{id}.md"))
}

/// Ids come from the UI, so only plain slugs may become file paths.
fn checked_path(config: &Config, id: &str) -> AppResult<PathBuf> {
    if !id.is_empty() && slug(id) == id {
        Ok(path(config, id))
    } else {
        Err(AppError::NotFound(format!("schedule {id}")))
    }
}

/// Reads every schedule file, sorted by name. A broken file never hides the others.
pub fn list(config: &Config) -> AppResult<Loaded> {
    let mut loaded = Loaded::default();
    for entry in std::fs::read_dir(dir(config))? {
        let file = entry?.path();
        if file.extension().is_some_and(|e| e == "md") {
            match read(&file) {
                Ok(s) => loaded.schedules.push(s),
                Err(e) => loaded.problems.push(format!("{}: {e}", file.display())),
            }
        }
    }
    loaded.schedules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(loaded)
}

pub fn find(config: &Config, id: &str) -> AppResult<Schedule> {
    read(&checked_path(config, id)?)
}

pub fn delete(config: &Config, id: &str) -> AppResult<()> {
    std::fs::remove_file(checked_path(config, id)?)?;
    Ok(())
}

fn read(file: &Path) -> AppResult<Schedule> {
    let id = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let text = std::fs::read_to_string(file)?;
    let (front, body) = frontmatter::split(&text)
        .ok_or_else(|| AppError::NotFound(format!("settings block (+++) in {id}.md")))?;
    let s: Settings = toml::from_str(front)?;
    Ok(Schedule {
        id,
        name: s.name,
        prompt: body.to_owned(),
        trigger: s.trigger,
        node: s.node,
        scope: s.scope,
        autonomy: s.autonomy,
        enabled: s.enabled,
        request: s.request,
    })
}

pub fn save(config: &Config, schedule: &Schedule) -> AppResult<()> {
    let settings = Settings {
        name: schedule.name.clone(),
        node: schedule.node.clone(),
        scope: schedule.scope,
        autonomy: schedule.autonomy,
        enabled: schedule.enabled,
        request: schedule.request.clone(),
        trigger: schedule.trigger.clone(),
    };
    let text = frontmatter::join(&toml::to_string(&settings)?, &schedule.prompt);
    std::fs::write(checked_path(config, &schedule.id)?, text)?;
    Ok(())
}

/// `base`, or `base-2`, `base-3`… if a schedule with that id already exists.
pub fn free_id(config: &Config, base: &str) -> String {
    (1_u32..=999)
        .map(|n| {
            if n == 1 {
                base.to_owned()
            } else {
                format!("{base}-{n}")
            }
        })
        .find(|id| !path(config, id).exists())
        .unwrap_or_else(|| base.to_owned())
}

/// A file-safe id from a name, e.g. "Morning brief" → "morning-brief".
pub fn slug(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words.is_empty() {
        "schedule".to_owned()
    } else {
        words.join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_names() {
        assert_eq!(slug("End-of-day wrap!"), "end-of-day-wrap");
        assert_eq!(slug("  "), "schedule");
    }

    #[test]
    fn refuses_ids_that_are_not_slugs() {
        let config = Config::open(std::env::temp_dir().join("bridge-test-config")).unwrap();
        assert!(checked_path(&config, "../bridge.toml").is_err());
        assert!(checked_path(&config, "").is_err());
        assert!(checked_path(&config, "morning-brief").is_ok());
    }
}
