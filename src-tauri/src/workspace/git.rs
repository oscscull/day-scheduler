//! Creating worktrees with `git`, and finding a PR's branch or a repo's default branch with `gh`.

use std::path::Path;

use tokio::process::Command;

use super::{Kind, Workspace};
use crate::cli;
use crate::error::{AppError, AppResult};

pub(super) async fn run(program: &str, dir: &Path, args: &[&str]) -> AppResult<String> {
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

/// The PR's head branch.
pub(super) async fn pr_branch(ws: &Workspace) -> AppResult<String> {
    let number = ws.number.to_string();
    let repo = ws.repo.as_str();
    let view = [
        "pr",
        "view",
        number.as_str(),
        "--repo",
        repo,
        "--json",
        "headRefName",
        "--jq",
        ".headRefName",
    ];
    run("gh", &ws.project, &view).await
}

pub(super) async fn default_branch(ws: &Workspace) -> AppResult<String> {
    let jq = ".defaultBranchRef.name";
    let view = [
        "repo",
        "view",
        ws.repo.as_str(),
        "--json",
        "defaultBranchRef",
        "--jq",
        jq,
    ];
    run("gh", &ws.project, &view).await
}

/// Makes the item's worktree, unless it exists.
pub async fn prepare(ws: &Workspace) -> AppResult<()> {
    if ws.dir.exists() {
        return Ok(());
    }
    match ws.kind {
        Kind::Pr => prepare_pr(ws).await?,
        Kind::Issue => prepare_issue(ws).await?,
    }
    copy_local_settings(ws)
}

/// At the PR's latest commit. The head is detached so the branch can stay checked out in the
/// user's own checkout; pushes go to `HEAD:<branch>`.
async fn prepare_pr(ws: &Workspace) -> AppResult<()> {
    let branch = pr_branch(ws).await?;
    run("git", &ws.project, &["fetch", "origin", branch.as_str()]).await?;
    let dir = ws.dir.display().to_string();
    let start = format!("origin/{branch}");
    let add = ["worktree", "add", "--detach", dir.as_str(), start.as_str()];
    run("git", &ws.project, &add).await.map(drop)
}

/// On the issue's own branch: reused if an earlier worktree left it, otherwise new from the latest
/// default branch, with no upstream so nothing pushes to the default branch by accident.
async fn prepare_issue(ws: &Workspace) -> AppResult<()> {
    let base = default_branch(ws).await?;
    run("git", &ws.project, &["fetch", "origin", base.as_str()]).await?;
    let dir = ws.dir.display().to_string();
    let branch = ws.issue_branch();
    let local = format!("refs/heads/{branch}");
    let verify = ["rev-parse", "--verify", "--quiet", local.as_str()];
    let exists = run("git", &ws.project, &verify).await.is_ok();
    let start = format!("origin/{base}");
    let (dir, branch, start) = (dir.as_str(), branch.as_str(), start.as_str());
    let add = if exists {
        vec!["worktree", "add", dir, branch]
    } else {
        vec!["worktree", "add", "--no-track", "-b", branch, dir, start]
    };
    run("git", &ws.project, &add).await.map(drop)
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
