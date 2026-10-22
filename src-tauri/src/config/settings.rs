use super::projects::Project;
use super::queue::QueueSettings;
use serde::{Deserialize, Serialize};

use super::Config;
use super::alerts::AlertSettings;
use super::calendar::CalendarSettings;
use super::dev::DevSettings;
use super::github::GithubSettings;
use super::sync::SyncSettings;
use super::updates::UpdateSettings;
use super::watchdog::WatchdogSettings;
use crate::error::{AppError, AppResult};
use crate::profile;

/// A part of life Quarterdeck keeps separate, e.g. Work, Personal projects, Life.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Area {
    pub id: String,
    pub name: String,
    /// Dot colour in the UI, as a CSS hex value.
    pub color: String,
    /// The machine that runs this area's schedules.
    pub node: String,
}

/// Mechanical settings from `quarterdeck.toml`. Anything missing falls back to a default, so a
/// hand-edited file with a line removed still loads.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub node: String,
    pub priority_limit: usize,
    /// How long something waits on reviewers before Today suggests a nudge.
    pub nudge_after_hours: i64,
    pub claude_command: String,
    pub activity_ttl_days: i64,
    pub run_ttl_days: i64,
    pub done_todo_ttl_days: i64,
    pub done_item_ttl_days: i64,
    pub areas: Vec<Area>,
    pub github: GithubSettings,
    pub calendar: CalendarSettings,
    pub alerts: AlertSettings,
    /// Start Quarterdeck when you log in, so schedules and alerts keep working.
    pub launch_at_login: bool,
    pub sync: SyncSettings,
    pub watchdog: WatchdogSettings,
    pub projects: Vec<Project>,
    /// Claude Code's `--permission-mode` for work in a project, e.g. `auto` (Claude decides what's
    /// safe, as in the user's own sessions) or `acceptEdits` (only the project's allow rules).
    pub project_permission_mode: String,
    pub queue: QueueSettings,
    pub updates: UpdateSettings,
    pub dev: DevSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            node: "this-mac".to_owned(),
            priority_limit: 3,
            nudge_after_hours: 24,
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
            sync: SyncSettings::default(),
            watchdog: WatchdogSettings::default(),
            projects: Vec::new(),
            project_permission_mode: "auto".to_owned(),
            queue: QueueSettings::default(),
            updates: UpdateSettings::default(),
            dev: DevSettings::default(),
        }
    }
}

impl Settings {
    /// The rules for deriving priorities, as of `now`.
    pub const fn priority_rules(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> crate::priorities::Rules {
        crate::priorities::Rules {
            now,
            nudge_after: chrono::Duration::hours(self.nudge_after_hours),
        }
    }

    /// The machine an area runs on; this machine for an unknown area.
    pub fn node_for(&self, area: &str) -> &str {
        self.areas
            .iter()
            .find(|a| a.id == area)
            .map_or(self.node.as_str(), |a| a.node.as_str())
    }

    /// The command that runs Claude. A dev build runs it only when `[dev] run_claude` is on.
    pub fn claude(&self) -> AppResult<&str> {
        if profile::DEV && !self.dev.run_claude {
            return Err(AppError::Claude(
                "this dev build doesn't run Claude; set [dev] run_claude = true in its quarterdeck.toml"
                    .to_owned(),
            ));
        }
        Ok(&self.claude_command)
    }

    /// `area` if it is configured (or none are), otherwise the first area.
    pub fn known_area(&self, area: &str) -> String {
        match self.areas.first() {
            Some(first) if !self.areas.iter().any(|a| a.id == area) => first.id.clone(),
            Some(_) | None => area.to_owned(),
        }
    }
}

pub fn load(config: &Config) -> AppResult<Settings> {
    let text = std::fs::read_to_string(config.root().join("quarterdeck.toml"))?;
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
            nudge_after_hours: 24,
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
            sync: SyncSettings::default(),
            watchdog: WatchdogSettings::default(),
            projects: Vec::new(),
            project_permission_mode: "auto".to_owned(),
            queue: QueueSettings::default(),
            updates: UpdateSettings::default(),
            dev: DevSettings::default(),
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
        assert_eq!(Settings::default().known_area("life"), "life");
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
            toml::from_str(include_str!("../../defaults/config/quarterdeck.toml")).unwrap();
        assert_eq!(parsed.areas.len(), 3);
    }
}
