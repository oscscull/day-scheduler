//! Runs headless Claude Code (`claude -p`) on the user's own subscription.

pub mod draft;
pub mod runner;

use std::path::PathBuf;

use tokio::process::Command;

use crate::error::{AppError, AppResult};

/// Apps opened from Finder don't get the shell's PATH, so look where Claude Code usually lives.
fn resolve(command: &str) -> PathBuf {
    let given = PathBuf::from(command);
    if given.is_absolute() || command.contains('/') {
        return given;
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let candidates = [
        home.join(".local/bin").join(command),
        home.join(".claude/local").join(command),
        PathBuf::from("/opt/homebrew/bin").join(command),
        PathBuf::from("/usr/local/bin").join(command),
    ];
    candidates.into_iter().find(|p| p.exists()).unwrap_or(given)
}

/// Sends one prompt and returns Claude's reply text. `tools` may be used without asking.
pub async fn ask_with(command: &str, prompt: &str, tools: &[String]) -> AppResult<String> {
    let mut cmd = Command::new(resolve(command));
    cmd.arg("-p").arg(prompt);
    if !tools.is_empty() {
        cmd.arg("--allowedTools").arg(tools.join(","));
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

/// Sends one prompt with no extra tools.
pub async fn ask(command: &str, prompt: &str) -> AppResult<String> {
    ask_with(command, prompt, &[]).await
}
