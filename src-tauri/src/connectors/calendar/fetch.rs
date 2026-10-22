use tokio::process::Command;

use crate::cli;
use crate::error::{AppError, AppResult};

/// Downloads a feed with the system's curl. `webcal://` links are fetched over https.
pub async fn ics(url: &str) -> AppResult<String> {
    let url = url
        .strip_prefix("webcal://")
        .map_or_else(|| url.to_owned(), |rest| format!("https://{rest}"));
    let out = Command::new(cli::resolve("curl"))
        .args(["-fsSL", "--max-time", "30", &url])
        .output()
        .await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(AppError::Connector(format!(
            "couldn't fetch a calendar: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}
