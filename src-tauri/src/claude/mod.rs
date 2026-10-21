//! Runs headless Claude Code (`claude -p`) on the user's own subscription.

pub mod draft;

use tokio::process::Command;

use crate::error::{AppError, AppResult};

/// Sends one prompt and returns Claude's reply text.
pub async fn ask(command: &str, prompt: &str) -> AppResult<String> {
    let out = Command::new(command).arg("-p").arg(prompt).output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(AppError::Claude(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ))
    }
}
