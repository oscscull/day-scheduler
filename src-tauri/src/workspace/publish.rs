//! After Claude finishes in a worktree, Quarterdeck does the GitHub side, since a project's sandbox
//! may keep Claude off the network: it pushes new commits and, for an issue, opens the PR.

use super::git::{default_branch, pr_branch, run};
use super::{Kind, Workspace};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Published {
    /// No commits to push.
    Nothing,
    /// New commits pushed to the PR's branch.
    Pushed,
    /// The issue's PR, opened now or found already open.
    Pr { number: u64, url: String },
}

pub async fn publish(ws: &Workspace) -> AppResult<Published> {
    match ws.kind {
        Kind::Pr => publish_pr(ws).await,
        Kind::Issue => publish_issue(ws).await,
    }
}

/// Commits on HEAD that `base` doesn't have.
async fn ahead_of(ws: &Workspace, base: &str) -> AppResult<u64> {
    let range = format!("{base}..HEAD");
    let count = run("git", &ws.dir, &["rev-list", "--count", range.as_str()]).await?;
    count
        .parse()
        .map_err(|_| AppError::Connector(format!("git rev-list --count gave '{count}'")))
}

async fn push(ws: &Workspace, branch: &str) -> AppResult<()> {
    let refspec = format!("HEAD:refs/heads/{branch}");
    run("git", &ws.dir, &["push", "origin", refspec.as_str()])
        .await
        .map(drop)
}

async fn publish_pr(ws: &Workspace) -> AppResult<Published> {
    let branch = pr_branch(ws).await?;
    run("git", &ws.dir, &["fetch", "origin", branch.as_str()]).await?;
    if ahead_of(ws, &format!("origin/{branch}")).await? == 0 {
        return Ok(Published::Nothing);
    }
    push(ws, &branch).await?;
    Ok(Published::Pushed)
}

async fn publish_issue(ws: &Workspace) -> AppResult<Published> {
    let base = default_branch(ws).await?;
    run("git", &ws.dir, &["fetch", "origin", base.as_str()]).await?;
    let base_ref = format!("origin/{base}");
    if ahead_of(ws, &base_ref).await? == 0 {
        return Ok(Published::Nothing);
    }
    let branch = ws.issue_branch();
    push(ws, &branch).await?;
    if let Some(open) = open_pr(ws, &branch).await? {
        return Ok(open);
    }
    let (title, body) = first_commit(ws, &base_ref).await?;
    let body = closing(&body, ws.number);
    let create = [
        "pr", "create", "--repo", &ws.repo, "--base", &base, "--head", &branch, "--title", &title,
        "--body", &body,
    ];
    let out = run("gh", &ws.project, &create).await?;
    pr_from_url(out.lines().last().unwrap_or_default())
}

/// The PR already open from `branch`, if any.
async fn open_pr(ws: &Workspace, branch: &str) -> AppResult<Option<Published>> {
    let list = [
        "pr", "list", "--repo", &ws.repo, "--head", branch, "--state", "open", "--json", "url",
        "--jq", ".[].url",
    ];
    let out = run("gh", &ws.project, &list).await?;
    out.lines().next().map(pr_from_url).transpose()
}

/// The subject and body of the branch's first commit, which Claude writes as the PR's.
async fn first_commit(ws: &Workspace, base_ref: &str) -> AppResult<(String, String)> {
    let range = format!("{base_ref}..HEAD");
    let shas = run("git", &ws.dir, &["log", "--reverse", "--format=%H", &range]).await?;
    let sha = shas.lines().next().unwrap_or("HEAD");
    let title = run("git", &ws.dir, &["log", "-1", "--format=%s", sha]).await?;
    let body = run("git", &ws.dir, &["log", "-1", "--format=%b", sha]).await?;
    Ok((title, body))
}

/// The body with "Closes #n" added, unless it already closes the issue.
fn closing(body: &str, number: u64) -> String {
    if already_closes(body, number) {
        return body.to_owned();
    }
    let line = format!("Closes #{number}");
    if body.trim().is_empty() {
        line
    } else {
        format!("{}\n\n{line}", body.trim_end())
    }
}

/// Whether the body has a closing keyword for `#number` itself, not e.g. `#1780` for 178.
fn already_closes(body: &str, number: u64) -> bool {
    let lower = body.to_lowercase();
    let keywords = [
        "close", "closes", "closed", "fix", "fixes", "fixed", "resolve", "resolves", "resolved",
    ];
    keywords.iter().any(|k| {
        let needle = format!("{k} #{number}");
        lower.match_indices(&needle).any(|(at, _)| {
            let next = lower
                .get(at + needle.len()..)
                .and_then(|rest| rest.chars().next());
            !next.is_some_and(|c| c.is_ascii_digit())
        })
    })
}

/// `https://github.com/o/r/pull/12` → PR 12.
fn pr_from_url(url: &str) -> AppResult<Published> {
    let url = url.trim();
    let number = url
        .rsplit_once("/pull/")
        .and_then(|(_, n)| n.parse().ok())
        .ok_or_else(|| AppError::Connector(format!("not a PR link: '{url}'")))?;
    Ok(Published::Pr {
        number,
        url: url.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_bodies_close_their_issue_once() {
        assert_eq!(closing("", 178), "Closes #178");
        assert_eq!(
            closing("Fixes the key.\n", 178),
            "Fixes the key.\n\nCloses #178"
        );
        assert_eq!(closing("Fixes #178.", 178), "Fixes #178.");
        assert_eq!(closing("Fixes #1780.", 178), "Fixes #1780.\n\nCloses #178");
    }

    #[test]
    fn reads_pr_links() {
        assert_eq!(
            pr_from_url("https://github.com/o/r/pull/12\n").unwrap(),
            Published::Pr {
                number: 12,
                url: "https://github.com/o/r/pull/12".to_owned()
            }
        );
        assert!(pr_from_url("https://github.com/o/r/issues/12").is_err());
    }
}
