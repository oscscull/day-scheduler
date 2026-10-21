//! Creating and removing worktrees with `git`, and finding a PR's branch with `gh`.

use std::path::Path;

use tokio::process::Command;

use super::Workspace;
use crate::cli;
use crate::error::{AppError, AppResult};

async fn run(program: &str, dir: &Path, args: &[&str]) -> AppResult<String> {
    let out = Command::new(cli::resolve(program))
        .current_dir(dir)
        .args(args)
        .output()
        .await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
        Err(AppError::Connector(format!(
            "{program} {}: {err}",
            args.join(" ")
        )))
    }
}

/// Makes the item's worktree at the PR's latest commit, unless it exists. The head is detached so
/// the branch can stay checked out in the user's own checkout; Claude pushes with `HEAD:<branch>`.
pub async fn prepare(ws: &Workspace) -> AppResult<()> {
    if ws.dir.exists() {
        return Ok(());
    }
    let number = ws.number.to_string();
    let view = [
        "pr",
        "view",
        number.as_str(),
        "--repo",
        ws.repo.as_str(),
        "--json",
        "headRefName",
        "--jq",
        ".headRefName",
    ];
    let branch = run("gh", &ws.project, &view).await?;
    run("git", &ws.project, &["fetch", "origin", branch.as_str()]).await?;
    let dir = ws.dir.display().to_string();
    let start = format!("origin/{branch}");
    run(
        "git",
        &ws.project,
        &["worktree", "add", "--detach", dir.as_str(), start.as_str()],
    )
    .await?;
    copy_local_settings(ws)
}

/// The user's personal Claude permissions for the project aren't in git, so bring them along.
fn copy_local_settings(ws: &Workspace) -> AppResult<()> {
    let local = Path::new(".claude").join("settings.local.json");
    let from = ws.project.join(&local);
    if from.exists() {
        let to = ws.dir.join(&local);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(from, to)?;
    }
    Ok(())
}
