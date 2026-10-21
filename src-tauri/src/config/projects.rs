//! `[[projects]]` in `quarterdeck.toml`: where a GitHub repo is checked out on this machine. Work Meatpuppet
//! starts on that repo's PRs runs there, in a worktree of its own.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    /// `owner/repo` on GitHub.
    pub repo: String,
    /// The checkout's folder. `~` means the home folder.
    pub path: String,
}

impl Project {
    /// The checkout folder with `~` expanded.
    pub fn dir(&self) -> PathBuf {
        match (self.path.strip_prefix("~/"), std::env::var_os("HOME")) {
            (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
            _ => PathBuf::from(&self.path),
        }
    }
}

/// The project for a repo, ignoring case as GitHub does.
pub fn find<'a>(projects: &'a [Project], repo: &str) -> Option<&'a Project> {
    projects.iter().find(|p| p.repo.eq_ignore_ascii_case(repo))
}
