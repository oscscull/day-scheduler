//! Runs headless Claude Code (`claude -p`) on the user's own subscription.

pub mod connect;
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

/// How to run one headless Claude call.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ask<'a> {
    /// Claude Code tools it may use without asking, besides the Bridge tools.
    pub tools: &'a [String],
    pub session: Session<'a>,
    /// The folder to run in. A resumed session must run where it started.
    pub cwd: Option<&'a str>,
    /// Claude Code's `--permission-mode`, e.g. `acceptEdits`. Commands still need the project's
    /// own permission rules.
    pub permission_mode: Option<&'a str>,
}

/// Which Claude Code session a run belongs to.
#[derive(Debug, Clone, Copy, Default)]
pub enum Session<'a> {
    /// A throwaway session.
    #[default]
    Fresh,
    /// A new session with this id, so Bridge can come back to it later.
    Start(&'a str),
    /// Continue an existing session with its own context.
    Resume(&'a str),
}

/// Sends one prompt and returns Claude's reply text. The Bridge tools are always available.
pub async fn ask_with(command: &str, prompt: &str, opts: &Ask<'_>) -> AppResult<String> {
    let mut allowed = vec![BRIDGE_TOOLS.to_owned()];
    allowed.extend_from_slice(opts.tools);
    let mut cmd = Command::new(cli::resolve(command));
    cmd.arg("-p")
        .arg(prompt)
        .arg("--mcp-config")
        .arg(bridge_server()?)
        .arg("--allowedTools")
        .arg(allowed.join(","));
    if let Some(dir) = opts.cwd {
        cmd.current_dir(dir);
    }
    if let Some(mode) = opts.permission_mode {
        cmd.arg("--permission-mode").arg(mode);
    }
    match opts.session {
        Session::Fresh => {}
        Session::Start(id) => drop(cmd.arg("--session-id").arg(id)),
        Session::Resume(id) => drop(cmd.arg("--resume").arg(id)),
    }
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
    ask_with(command, prompt, &Ask::default()).await
}
