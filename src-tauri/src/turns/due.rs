//! What an item's clock calls for. Pure, so the rules are easy to test.

use chrono::{DateTime, Utc};

use crate::config::turns::TurnSettings;
use crate::domain::item::{Stage, WorkItem};
use crate::domain::turn::Turn;

/// What Quarterdeck knows about the agent's work on an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Work {
    /// A run about the item is going.
    Running,
    /// An answer or message is waiting to go to the agent.
    Pending,
    /// A session outside Quarterdeck was heard from lately.
    Elsewhere,
    /// Nothing is happening.
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Due {
    Wait,
    /// In flight with no clock yet: start one.
    StartClock,
    /// The agent's turn and nothing is running: prompt it.
    Prompt,
    /// The user's turn ran out: remind them, and start the clock again.
    Remind,
    /// The turn ran out: pass it on.
    PassTo {
        to: Turn,
        why: String,
    },
}

const fn in_flight(item: &WorkItem) -> bool {
    matches!(item.stage, Stage::InProgress | Stage::InReview)
}

fn agent(work: Work, late: bool) -> Due {
    if late {
        let why = if work == Work::Running {
            "the agent's turn ran out while it was still running"
        } else {
            "the agent's turn ran out"
        };
        return Due::PassTo {
            to: Turn::You,
            why: why.to_owned(),
        };
    }
    match work {
        Work::Idle => Due::Prompt,
        Work::Running | Work::Pending | Work::Elsewhere => Due::Wait,
    }
}

fn others(item: &WorkItem, limits: &TurnSettings) -> Due {
    let who = item.waiting_on.as_deref().unwrap_or("others");
    Due::PassTo {
        to: item.then_turn.unwrap_or(limits.others_then),
        why: format!("waited {}h on {who}", limits.others_hours),
    }
}

pub fn due(item: &WorkItem, work: Work, limits: &TurnSettings, now: DateTime<Utc>) -> Due {
    if !in_flight(item) {
        return Due::Wait;
    }
    let Some(until) = item.turn_until else {
        return Due::StartClock;
    };
    let late = now >= until;
    match item.turn {
        Turn::Agent => agent(work, late),
        _ if !late => Due::Wait,
        Turn::You => Due::Remind,
        Turn::Others => others(item, limits),
        Turn::Later => Due::PassTo {
            to: item.then_turn.unwrap_or(Turn::You),
            why: "its time came".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn item(turn: Turn, until: Option<i64>) -> WorkItem {
        let now = Utc::now();
        WorkItem {
            id: "a".to_owned(),
            title: String::new(),
            state_label: String::new(),
            stage: Stage::InReview,
            status: None,
            turn,
            turn_until: until.map(|m| now + Duration::minutes(m)),
            waiting_on: Some("CI".to_owned()),
            then_turn: None,
            area: String::new(),
            link: None,
            source: String::new(),
            external_id: None,
            opened_at: now,
            state_since: now,
            rank: 0,
        }
    }

    fn check(i: &WorkItem, work: Work) -> Due {
        due(i, work, &TurnSettings::default(), Utc::now())
    }

    const IDLE: Work = Work::Idle;

    #[test]
    fn nothing_in_flight_idles_without_a_clock() {
        assert_eq!(check(&item(Turn::You, None), IDLE), Due::StartClock);
        let mut later = item(Turn::You, Some(-5));
        later.stage = Stage::UpNext;
        assert_eq!(check(&later, IDLE), Due::Wait);
    }

    #[test]
    fn the_agent_is_prompted_until_its_time_runs_out() {
        assert_eq!(check(&item(Turn::Agent, Some(60)), IDLE), Due::Prompt);
        let running = Work::Running;
        assert_eq!(check(&item(Turn::Agent, Some(60)), running), Due::Wait);
        assert_eq!(
            check(&item(Turn::Agent, Some(60)), Work::Elsewhere),
            Due::Wait
        );
        assert!(matches!(
            check(&item(Turn::Agent, Some(-1)), running),
            Due::PassTo { to: Turn::You, .. }
        ));
    }

    #[test]
    fn waits_end_and_the_user_is_reminded() {
        assert_eq!(check(&item(Turn::You, Some(-1)), IDLE), Due::Remind);
        assert_eq!(check(&item(Turn::You, Some(60)), IDLE), Due::Wait);
        assert_eq!(
            check(&item(Turn::Others, Some(-1)), IDLE),
            Due::PassTo {
                to: Turn::You,
                why: "waited 24h on CI".to_owned()
            }
        );
        let mut to_agent = item(Turn::Others, Some(-1));
        to_agent.then_turn = Some(Turn::Agent);
        assert!(matches!(
            check(&to_agent, IDLE),
            Due::PassTo {
                to: Turn::Agent,
                ..
            }
        ));
    }
}
