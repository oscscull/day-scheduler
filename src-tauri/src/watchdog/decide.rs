//! Whether an item Claude holds needs a check-in. Pure, so the rules are easy to test.

use chrono::{DateTime, Duration, Utc};

use crate::domain::item::{Stage, Status, WorkItem};
use crate::store::watch::Watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    /// Ask Claude for an update.
    Nudge,
    /// Nudges went unanswered; tell the user instead.
    Escalate,
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub quiet: Duration,
    pub max_nudges: u32,
}

/// Items Claude is carrying right now. Queued and finished work isn't watched.
pub fn held_by_claude(item: &WorkItem) -> bool {
    matches!(item.stage, Stage::InProgress | Stage::InReview)
        && (matches!(item.status, Some(Status::Proceeding | Status::CiFailed))
            || item.holder.eq_ignore_ascii_case("claude"))
}

/// The watch with nudges cleared if Claude has spoken since the last one.
pub fn refreshed(watch: &Watch, last_heard: DateTime<Utc>) -> Watch {
    match watch.last_nudge {
        Some(at) if last_heard > at => Watch {
            nudges: 0,
            last_nudge: None,
            ..watch.clone()
        },
        _ => watch.clone(),
    }
}

pub fn decide(
    last_heard: DateTime<Utc>,
    watch: &Watch,
    now: DateTime<Utc>,
    limits: Limits,
) -> Action {
    let quiet_since_update = now - last_heard >= limits.quiet;
    let quiet_since_nudge = watch.last_nudge.is_none_or(|at| now - at >= limits.quiet);
    if !(quiet_since_update && quiet_since_nudge) {
        return Action::Wait;
    }
    match watch.nudges.cmp(&limits.max_nudges) {
        std::cmp::Ordering::Less => Action::Nudge,
        std::cmp::Ordering::Equal => Action::Escalate,
        std::cmp::Ordering::Greater => Action::Wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: Limits = Limits {
        quiet: Duration::minutes(45),
        max_nudges: 2,
    };

    fn ago(m: i64) -> DateTime<Utc> {
        Utc::now() - Duration::minutes(m)
    }

    #[test]
    fn recent_updates_mean_wait() {
        assert_eq!(
            decide(ago(10), &Watch::default(), Utc::now(), LIMITS),
            Action::Wait
        );
    }

    #[test]
    fn silence_nudges_then_escalates_then_stops() {
        let quiet = ago(120);
        assert_eq!(
            decide(quiet, &Watch::default(), Utc::now(), LIMITS),
            Action::Nudge
        );
        let once = Watch {
            nudges: 1,
            last_nudge: Some(ago(10)),
            ..Watch::default()
        };
        assert_eq!(decide(quiet, &once, Utc::now(), LIMITS), Action::Wait);
        let twice = Watch {
            nudges: 2,
            last_nudge: Some(ago(50)),
            ..Watch::default()
        };
        assert_eq!(decide(quiet, &twice, Utc::now(), LIMITS), Action::Escalate);
        let told = Watch {
            nudges: 3,
            last_nudge: Some(ago(50)),
            ..Watch::default()
        };
        assert_eq!(decide(quiet, &told, Utc::now(), LIMITS), Action::Wait);
    }

    #[test]
    fn hearing_back_resets_the_count() {
        let w = Watch {
            nudges: 2,
            last_nudge: Some(ago(60)),
            ..Watch::default()
        };
        assert_eq!(refreshed(&w, ago(5)).nudges, 0);
        assert_eq!(refreshed(&w, ago(90)).nudges, 2);
    }
}
