//! What Bridge derives from a PR: the item fields it owns, and the signals that fire events.

use serde::{Deserialize, Serialize};

use super::fetch::Pr;
use crate::domain::item::{Stage, Status};
use crate::domain::schedule::EventKind;

/// The item fields GitHub decides. They are only re-applied when they change upstream, so edits
/// by the user or an agent survive until GitHub says something new.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owned {
    pub title: String,
    pub stage: Stage,
    pub status: Option<Status>,
    pub label: String,
    pub holder: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub owned: Owned,
    pub ci_failing: bool,
    pub comments: u64,
}

fn failing(pr: &Pr) -> bool {
    matches!(pr.ci.as_deref(), Some("FAILURE" | "ERROR"))
}

/// (status, label, holder) for a PR, most pressing first.
fn state(pr: &Pr) -> (Option<Status>, &'static str, &'static str) {
    if failing(pr) {
        return (Some(Status::Ci), "CI failing", "CI");
    }
    if pr.draft {
        return (None, "draft", "you");
    }
    match pr.review.as_deref() {
        Some("CHANGES_REQUESTED") => (None, "changes requested", "you"),
        Some("APPROVED") => (None, "approved, ready to merge", "you"),
        _ => (Some(Status::Review), "waiting on review", "reviewers"),
    }
}

pub fn of(pr: &Pr) -> Snapshot {
    let (status, label, holder) = state(pr);
    Snapshot {
        owned: Owned {
            title: pr.title.clone(),
            stage: if pr.draft {
                Stage::InProgress
            } else {
                Stage::InReview
            },
            status,
            label: label.to_owned(),
            holder: holder.to_owned(),
        },
        ci_failing: failing(pr),
        comments: pr.comments,
    }
}

/// Events since the last snapshot. Nothing fires the first time a PR is seen.
pub fn events(old: Option<&Snapshot>, new: &Snapshot) -> Vec<EventKind> {
    let Some(old) = old else { return Vec::new() };
    let mut events = Vec::new();
    if new.comments > old.comments {
        events.push(EventKind::PrNewComments);
    }
    if new.ci_failing && !old.ci_failing {
        events.push(EventKind::CiFailed);
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(ci: Option<&str>, review: Option<&str>, comments: u64) -> Pr {
        Pr {
            repo: "me/app".to_owned(),
            number: 1,
            title: "Fix".to_owned(),
            url: String::new(),
            draft: false,
            review: review.map(str::to_owned),
            ci: ci.map(str::to_owned),
            comments,
        }
    }

    #[test]
    fn failing_ci_wins_over_review_state() {
        let s = of(&pr(Some("FAILURE"), Some("APPROVED"), 0));
        assert_eq!(s.owned.status, Some(Status::Ci));
    }

    #[test]
    fn events_fire_on_change_only() {
        let before = of(&pr(Some("SUCCESS"), None, 2));
        let after = of(&pr(Some("FAILURE"), None, 3));
        assert_eq!(events(None, &after), []);
        assert_eq!(events(Some(&before), &before), []);
        assert_eq!(
            events(Some(&before), &after),
            [EventKind::PrNewComments, EventKind::CiFailed]
        );
    }
}
