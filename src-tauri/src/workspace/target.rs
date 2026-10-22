//! What an item's work is on GitHub: a PR (from a GitHub item's id) or an issue (from its link).

use crate::connectors::github;
use crate::domain::item::WorkItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// An open PR: the worktree sits at its head.
    Pr,
    /// An issue to fix: the worktree is a new branch off the default branch.
    Issue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// `owner/repo`.
    pub repo: String,
    pub number: u64,
    pub kind: Kind,
}

/// The PR a GitHub item stands for, from its external id `owner/repo#12`.
fn pr_of(item: &WorkItem) -> Option<Target> {
    if item.source != github::SOURCE {
        return None;
    }
    let (repo, number) = item.external_id.as_deref()?.split_once('#')?;
    Some(Target {
        repo: repo.to_owned(),
        number: number.parse().ok()?,
        kind: Kind::Pr,
    })
}

/// The issue an item links to: `https://github.com/<owner>/<repo>/issues/<n>`, with any trailing
/// `/`, `#…` or `?…` ignored.
fn issue_of(item: &WorkItem) -> Option<Target> {
    let link = item.link.as_deref()?.trim();
    let path = link.strip_prefix("https://github.com/")?;
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    if parts.next()? != "issues" || owner.is_empty() || repo.is_empty() {
        return None;
    }
    let digits: String = parts
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    Some(Target {
        repo: format!("{owner}/{repo}"),
        number: digits.parse().ok()?,
        kind: Kind::Issue,
    })
}

/// A PR item's PR, else the issue the item links to.
pub fn of(item: &WorkItem) -> Option<Target> {
    pr_of(item).or_else(|| issue_of(item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::Stage;
    use chrono::Utc;

    fn item(source: &str, external_id: Option<&str>, link: Option<&str>) -> WorkItem {
        WorkItem {
            id: "x#1".to_owned(),
            title: "t".to_owned(),
            state_label: String::new(),
            stage: Stage::UpNext,
            status: None,
            turn: crate::domain::turn::Turn::You,
            turn_until: None,
            waiting_on: None,
            then_turn: None,
            area: "work".to_owned(),
            link: link.map(str::to_owned),
            source: source.to_owned(),
            external_id: external_id.map(str::to_owned),
            opened_at: Utc::now(),
            state_since: Utc::now(),
            rank: 0,
        }
    }

    fn target(repo: &str, number: u64, kind: Kind) -> Target {
        Target {
            repo: repo.to_owned(),
            number,
            kind,
        }
    }

    #[test]
    fn github_items_are_their_pr() {
        let pr = item(
            "github",
            Some("me/app#7"),
            Some("https://github.com/me/app/pull/7"),
        );
        assert_eq!(of(&pr), Some(target("me/app", 7, Kind::Pr)));
    }

    #[test]
    fn items_linking_an_issue_are_that_issue() {
        for link in [
            "https://github.com/oscscull/baseddsl/issues/178",
            "https://github.com/oscscull/baseddsl/issues/178/",
            "https://github.com/oscscull/baseddsl/issues/178#issuecomment-1",
            " https://github.com/oscscull/baseddsl/issues/178?x=1 ",
        ] {
            let issue = item("manual", None, Some(link));
            assert_eq!(
                of(&issue),
                Some(target("oscscull/baseddsl", 178, Kind::Issue)),
                "{link}"
            );
        }
    }

    #[test]
    fn other_links_have_no_target() {
        for link in [
            "https://github.com/me/app/pull/7",
            "https://github.com/me/app",
            "https://github.com/me/app/issues/",
            "https://example.com/me/app/issues/3",
            "https://linear.app/x/issue/AB-1",
        ] {
            assert_eq!(of(&item("manual", None, Some(link))), None, "{link}");
        }
        assert_eq!(of(&item("manual", None, None)), None);
    }
}
