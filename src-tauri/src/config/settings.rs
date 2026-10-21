use serde::{Deserialize, Serialize};

use super::Config;
use super::alerts::AlertSettings;
use super::calendar::CalendarSettings;
use super::github::GithubSettings;
use crate::error::AppResult;

/// A part of life Bridge keeps separate, e.g. Work, Personal projects, Life.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Area {
    pub id: String,
    pub name: String,
    /// Dot colour in the UI, as a CSS hex value.
    pub color: String,
    /// The machine that runs this area's schedules.
    pub node: String,
}

/// Mechanical settings from `bridge.toml`. Anything missing falls back to a default, so a
/// hand-edited file with a line removed still loads.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub node: String,
    pub priority_limit: usize,
    pub claude_command: String,
    pub activity_ttl_days: i64,
    pub run_ttl_days: i64,
    pub done_todo_ttl_days: i64,
    pub done_item_ttl_days: i64,
    pub areas: Vec<Area>,
    pub github: GithubSettings,
    pub calendar: CalendarSettings,
    pub alerts: AlertSettings,
    /// Start Bridge when you log in, so schedules and alerts keep working.
    pub launch_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            node: "this-mac".to_owned(),
            priority_limit: 3,
            claude_command: "claude".to_owned(),
            activity_ttl_days: 7,
            run_ttl_days: 14,
            done_todo_ttl_days: 1,
            done_item_ttl_days: 30,
            areas: Vec::new(),
            github: GithubSettings::default(),
            calendar: CalendarSettings::default(),
            alerts: AlertSettings::default(),
            launch_at_login: true,
        }
    }
}

impl Settings {
    /// The machine an area runs on; this machine for an unknown area.
    pub fn node_for(&self, area: &str) -> &str {
        self.areas
            .iter()
            .find(|a| a.id == area)
            .map_or(self.node.as_str(), |a| a.node.as_str())
    }

    /// `area` if it is configured, otherwise the first area.
    pub fn known_area(&self, area: &str) -> String {
        let fallback = self
            .areas
            .first()
            .map_or_else(String::new, |a| a.id.clone());
        if self.areas.iter().any(|a| a.id == area) {
            area.to_owned()
        } else {
            fallback
        }
    }
}

pub fn load(config: &Config) -> AppResult<Settings> {
    let text = std::fs::read_to_string(config.root().join("bridge.toml"))?;
    Ok(toml::from_str(&text)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        let area = |id: &str, node: &str| Area {
            id: id.to_owned(),
            name: id.to_owned(),
            color: String::new(),
            node: node.to_owned(),
        };
        Settings {
            node: "this-mac".to_owned(),
            priority_limit: 3,
            claude_command: String::new(),
            activity_ttl_days: 7,
            run_ttl_days: 14,
            done_todo_ttl_days: 1,
            done_item_ttl_days: 30,
            areas: vec![area("work", "work-mac"), area("life", "home-mac")],
            github: GithubSettings::default(),
            calendar: CalendarSettings::default(),
            alerts: AlertSettings::default(),
            launch_at_login: true,
        }
    }

    #[test]
    fn areas_pick_their_machine() {
        assert_eq!(settings().node_for("life"), "home-mac");
        assert_eq!(settings().node_for("nope"), "this-mac");
    }

    #[test]
    fn unknown_areas_fall_back_to_the_first() {
        assert_eq!(settings().known_area("life"), "life");
        assert_eq!(settings().known_area("nope"), "work");
    }

    #[test]
    fn missing_lines_fall_back() {
        let parsed: Settings = toml::from_str("node = \"x\"").unwrap();
        assert_eq!(parsed.priority_limit, 3);
        assert_eq!(parsed.node, "x");
    }

    #[test]
    fn shipped_settings_parse() {
        let parsed: Settings =
            toml::from_str(include_str!("../../defaults/config/bridge.toml")).unwrap();
        assert_eq!(parsed.areas.len(), 3);
    }
}
