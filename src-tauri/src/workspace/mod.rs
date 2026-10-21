//! Work Quarterdeck starts on a PR runs in that repo's project, in a git worktree of its own per item.
//! The user's checkout is never touched, and the project's own Claude settings apply. A worktree
//! lives while its item is open.

mod git;
mod sweep;

use std::path::PathBuf;

pub use git::prepare;
pub use sweep::sweep;

use crate::config::Config;
use crate::config::projects;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::connectors::github;
use crate::domain::item::WorkItem;
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::paths;

/// Where one item's work happens.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// The user's checkout of the repo.
    pub project: PathBuf,
    pub repo: String,
    pub number: u64,
    /// The item's own worktree.
    pub dir: PathBuf,
}

impl Workspace {
    /// A new session that will run in this worktree.
    pub fn new_session(&self) -> SessionRef {
        SessionRef {
            id: uuid::Uuid::new_v4().to_string(),
            cwd: Some(self.dir.display().to_string()),
        }
    }

    pub fn holds(&self, session: &SessionRef) -> bool {
        session.cwd.as_deref().map(PathBuf::from) == Some(self.dir.clone())
    }
}

fn root() -> AppResult<PathBuf> {
    Ok(paths::data_dir()?.join("worktrees"))
}

/// A folder name for an item id, e.g. `app#12` → `app-12`.
fn folder(item_id: &str) -> String {
    item_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// `owner/repo` and the PR number from a GitHub item's external id, `owner/repo#12`.
fn pr_of(item: &WorkItem) -> Option<(&str, u64)> {
    if item.source != github::SOURCE {
        return None;
    }
    let (repo, number) = item.external_id.as_deref()?.split_once('#')?;
    Some((repo, number.parse().ok()?))
}

/// The workspace for an item, if it's a PR in a repo with a project on this machine.
pub fn for_item(settings: &Settings, item: &WorkItem) -> AppResult<Option<Workspace>> {
    let Some((repo, number)) = pr_of(item) else {
        return Ok(None);
    };
    let Some(project) = projects::find(&settings.projects, repo) else {
        return Ok(None);
    };
    Ok(Some(Workspace {
        project: project.dir(),
        repo: repo.to_owned(),
        number,
        dir: root()?.join(folder(&item.id)),
    }))
}

/// Tells a run where it is: in a worktree it can push from, or with no checkout at all.
pub fn describe(config: &Config, ws: Option<&Workspace>) -> AppResult<String> {
    ws.map_or_else(
        || render(config, Prompt::NoWorkspace, &[]),
        |ws| {
            let number = ws.number.to_string();
            let vars = [("repo", ws.repo.as_str()), ("number", number.as_str())];
            render(config, Prompt::Workspace, &vars)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_are_safe_names() {
        assert_eq!(folder("labbase-webapp#1928"), "labbase-webapp-1928");
    }
}
