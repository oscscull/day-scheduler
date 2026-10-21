//! Whether the user's own Claude Code sessions can reach Quarterdeck, and the command that connects them.

use std::path::Path;

use serde::Serialize;

use crate::error::AppResult;
use crate::paths;

/// Quarterdeck's registration in Claude Code's user config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Connection {
    Connected,
    /// Not registered; run `command` once.
    Missing {
        command: String,
    },
    /// Registered, but for another copy of the app (e.g. a dev build); `command` re-points it.
    Elsewhere {
        command: String,
    },
}

/// The server name Quarterdeck registers under, and the name it had before (Bridge).
const NAME: &str = "quarterdeck";
const OLD_NAME: &str = "bridge";

/// The `command` Claude Code runs for the server called `name`, if registered at user scope.
fn registered(config: &serde_json::Value, name: &str) -> Option<String> {
    let command = config.get("mcpServers")?.get(name)?.get("command")?;
    command.as_str().map(str::to_owned)
}

fn add(exe: &Path) -> String {
    format!(
        "claude mcp add --scope user {NAME} -- \"{}\" mcp",
        exe.display()
    )
}

fn classify(config: Option<&str>, exe: &Path) -> Connection {
    let json: serde_json::Value = config
        .and_then(|c| serde_json::from_str(c).ok())
        .unwrap_or_default();
    let current = registered(&json, NAME);
    let old = registered(&json, OLD_NAME);
    if current.as_deref().is_some_and(|c| Path::new(c) == exe) && old.is_none() {
        return Connection::Connected;
    }
    let stale: Vec<String> = [(NAME, &current), (OLD_NAME, &old)]
        .iter()
        .filter(|(_, command)| command.is_some())
        .map(|(name, _)| format!("claude mcp remove --scope user {name} && "))
        .collect();
    if stale.is_empty() {
        return Connection::Missing { command: add(exe) };
    }
    Connection::Elsewhere {
        command: format!("{}{}", stale.concat(), add(exe)),
    }
}

/// Reads Claude Code's config fresh each time, so running the command shows up at once.
pub fn status() -> AppResult<Connection> {
    let exe = std::env::current_exe()?;
    let config = std::fs::read_to_string(paths::claude_config()?).ok();
    Ok(classify(config.as_deref(), &exe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_only_when_registered_for_this_binary() {
        let exe = Path::new("/apps/quarterdeck");
        let this =
            r#"{"mcpServers":{"quarterdeck":{"command":"/apps/quarterdeck","args":["mcp"]}}}"#;
        let renamed = r#"{"mcpServers":{"bridge":{"command":"/apps/day-scheduler"}}}"#;
        let other = r#"{"mcpServers":{"quarterdeck":{"command":"/old/quarterdeck"}}}"#;
        assert_eq!(classify(Some(this), exe), Connection::Connected);
        assert!(matches!(
            classify(Some(other), exe),
            Connection::Elsewhere { .. }
        ));
        assert!(matches!(
            classify(Some("{}"), exe),
            Connection::Missing { .. }
        ));
        assert!(matches!(classify(None, exe), Connection::Missing { .. }));
        let Connection::Elsewhere { command } = classify(Some(renamed), exe) else {
            panic!("an old Bridge registration should be replaced");
        };
        assert!(command.starts_with("claude mcp remove --scope user bridge && claude mcp add"));
    }
}
