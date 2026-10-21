//! Whether the user's own Claude Code sessions can reach Bridge, and the command that connects them.

use std::path::Path;

use serde::Serialize;

use crate::error::AppResult;
use crate::paths;

/// Bridge's registration in Claude Code's user config.
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

/// The `command` Claude Code runs for the `bridge` server, if registered at user scope.
fn registered(config: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(config).ok()?;
    let command = json.get("mcpServers")?.get("bridge")?.get("command")?;
    command.as_str().map(str::to_owned)
}

fn add(exe: &Path) -> String {
    format!(
        "claude mcp add --scope user bridge -- \"{}\" mcp",
        exe.display()
    )
}

fn classify(config: Option<&str>, exe: &Path) -> Connection {
    match config.and_then(registered) {
        Some(command) if Path::new(&command) == exe => Connection::Connected,
        Some(_) => Connection::Elsewhere {
            command: format!("claude mcp remove --scope user bridge && {}", add(exe)),
        },
        None => Connection::Missing { command: add(exe) },
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
        let exe = Path::new("/apps/bridge");
        let this = r#"{"mcpServers":{"bridge":{"command":"/apps/bridge","args":["mcp"]}}}"#;
        let other = r#"{"mcpServers":{"bridge":{"command":"/old/bridge"}}}"#;
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
    }
}
