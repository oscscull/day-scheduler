//! The `[github]` section of `bridge.toml`.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GithubSettings {
    /// Uses the `gh` command's existing login.
    pub enabled: bool,
    pub poll_minutes: i64,
    /// Area for PRs from repos not listed in `areas`.
    pub default_area: String,
    /// Owner (`my-org`) or repo (`my-org/app`) to area id. A repo entry beats its owner's.
    pub areas: BTreeMap<String, String>,
    /// Owners or repos whose PRs are skipped entirely.
    pub ignore: Vec<String>,
}

impl Default for GithubSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_minutes: 5,
            default_area: "work".to_owned(),
            areas: BTreeMap::new(),
            ignore: Vec::new(),
        }
    }
}

fn owner(repo: &str) -> &str {
    repo.split_once('/').map_or(repo, |(o, _)| o)
}

impl GithubSettings {
    pub fn area_for(&self, repo: &str) -> &str {
        self.areas
            .get(repo)
            .or_else(|| self.areas.get(owner(repo)))
            .unwrap_or(&self.default_area)
    }

    pub fn ignores(&self, repo: &str) -> bool {
        self.ignore.iter().any(|i| i == repo || i == owner(repo))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_beats_owner_beats_default() {
        let g: GithubSettings = toml::from_str(
            "default_area = \"work\"\nignore = [\"spam\"]\n[areas]\nme = \"projects\"\n\"me/day-job\" = \"work\"",
        )
        .unwrap();
        assert_eq!(g.area_for("me/day-job"), "work");
        assert_eq!(g.area_for("me/toy"), "projects");
        assert_eq!(g.area_for("acme/app"), "work");
        assert!(g.ignores("spam/thing"));
    }
}
