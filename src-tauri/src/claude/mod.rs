//! Runs headless Claude Code (`claude -p`) on the user's own subscription.

pub mod draft;
pub mod runner;

use tokio::process::Command;

use crate::cli;
use crate::error::{AppError, AppResult};

/// Lets a headless run use every Bridge tool without asking.
const BRIDGE_TOOLS: &str = "mcp__bridge";

/// The Bridge MCP server for a headless run: this same binary, started with `mcp`.
fn bridge_server() -> AppResult<String> {
    let exe = std::env::current_exe()?;
    let config = serde_json::json!({
        "mcpServers": { "bridge": { "command": exe, "args": ["mcp"] } }
    });
    Ok(config.to_string())
}

/// Sends one prompt and returns Claude's reply text. The Bridge tools are always available;
/// `tools` lists other Claude Code tools it may use without asking.
pub async fn ask_with(command: &str, prompt: &str, tools: &[String]) -> AppResult<String> {
    let mut allowed = vec![BRIDGE_TOOLS.to_owned()];
    allowed.extend_from_slice(tools);
    let mut cmd = Command::new(cli::resolve(command));
    cmd.arg("-p")
        .arg(prompt)
        .arg("--mcp-config")
        .arg(bridge_server()?)
        .arg("--allowedTools")
        .arg(allowed.join(","));
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(AppError::Claude(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ))
    }
}

/// Sends one prompt with only the Bridge tools.
pub async fn ask(command: &str, prompt: &str) -> AppResult<String> {
    ask_with(command, prompt, &[]).await
}

/// The command a user runs once so their own Claude Code sessions can use Bridge.
pub fn connect_command() -> AppResult<String> {
    let exe = std::env::current_exe()?;
    Ok(format!(
        "claude mcp add --scope user bridge -- \"{}\" mcp",
        exe.display()
    ))
}
