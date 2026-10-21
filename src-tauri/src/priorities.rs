//! What needs the user, derived from the work itself and ranked by it. Never ordered by hand.
//!
//! First, in rank order: explicit asks and items waiting on the user's decision. Then things an
//! agent should be fixing (failing CI). Lowest: nudges for items that have waited on reviewers
//! too long.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::domain::ask::Ask;
use crate::domain::item::{Stage, Status, WorkItem};

/// Why something is a priority. The UI words each kind from the copy file.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reason {
    /// A question or design fork someone raised on the item.
    Ask { ask: Ask },
    /// The item itself waits on the user, e.g. a PR to review or merge.
    Decision,
    /// Something broke that an agent should fix, e.g. failing CI.
    Fix,
    /// The item has waited on others long enough to chase.
    Nudge,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    #[serde(flatten)]
    pub reason: Reason,
    pub item_id: String,
    pub item_title: String,
    pub item_label: String,
    pub holder: String,
    pub area: String,
    /// When the item reached its current state.
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rules {
    pub now: DateTime<Utc>,
    /// How long an item waits on reviewers before it's worth a nudge.
    pub nudge_after: Duration,
}

fn priority(item: &WorkItem, reason: Reason) -> Priority {
    Priority {
        reason,
        item_id: item.id.clone(),
        item_title: item.title.clone(),
        item_label: item.state_label.clone(),
        holder: item.holder.clone(),
        area: item.area.clone(),
        since: item.state_since,
    }
}

/// Asks on the item, or the item itself when it waits on the user's decision.
fn needs_you(item: &WorkItem, asks: &[Ask]) -> Vec<Priority> {
    let mine: Vec<Priority> = asks
        .iter()
        .filter(|a| a.item_id == item.id)
        .map(|a| priority(item, Reason::Ask { ask: a.clone() }))
        .collect();
    if mine.is_empty() && item.status == Some(Status::Decision) {
        return vec![priority(item, Reason::Decision)];
    }
    mine
}

fn worth_a_nudge(item: &WorkItem, rules: Rules) -> bool {
    item.status == Some(Status::Review) && rules.now - item.state_since >= rules.nudge_after
}

/// Every priority, in order. `items` must already be sorted by rank.
pub fn priorities(items: &[WorkItem], asks: &[Ask], rules: Rules) -> Vec<Priority> {
    let open: Vec<&WorkItem> = items.iter().filter(|i| i.stage != Stage::Done).collect();
    let first = open.iter().flat_map(|i| needs_you(i, asks));
    let broken = open
        .iter()
        .filter(|i| i.status == Some(Status::CiFailed))
        .map(|i| priority(i, Reason::Fix));
    let later = open
        .iter()
        .filter(|i| worth_a_nudge(i, rules))
        .map(|i| priority(i, Reason::Nudge));
    first.chain(broken).chain(later).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, status: Option<Status>, waited_hours: i64) -> WorkItem {
        let now = Utc::now();
        WorkItem {
            id: id.to_owned(),
            title: id.to_owned(),
            state_label: String::new(),
            stage: Stage::InReview,
            status,
            holder: String::new(),
            area: String::new(),
            link: None,
            source: String::new(),
            external_id: None,
            opened_at: now,
            state_since: now - Duration::hours(waited_hours),
            rank: 0,
        }
    }

    fn ask(id: i64, item_id: &str) -> Ask {
        Ask {
            id,
            item_id: item_id.to_owned(),
            question: String::new(),
            context: String::new(),
        }
    }

    const RULES: fn() -> Rules = || Rules {
        now: Utc::now(),
        nudge_after: Duration::hours(24),
    };

    fn kinds(list: &[Priority]) -> Vec<String> {
        list.iter()
            .map(|p| match &p.reason {
                Reason::Ask { ask } => format!("ask{}", ask.id),
                Reason::Decision => format!("decide {}", p.item_id),
                Reason::Fix => format!("fix {}", p.item_id),
                Reason::Nudge => format!("nudge {}", p.item_id),
            })
            .collect()
    }

    #[test]
    fn asks_and_decisions_come_in_rank_order_then_nudges() {
        let items = [
            item("stale", Some(Status::Review), 48),
            item("mine", Some(Status::Decision), 1),
            item("asked", None, 1),
            item("fresh", Some(Status::Review), 2),
            item("broken", Some(Status::CiFailed), 1),
        ];
        let asks = [ask(7, "asked")];
        assert_eq!(
            kinds(&priorities(&items, &asks, RULES())),
            ["decide mine", "ask7", "fix broken", "nudge stale"]
        );
    }

    #[test]
    fn an_ask_stands_in_for_the_decision_itself() {
        let items = [item("mine", Some(Status::Decision), 1)];
        assert_eq!(
            kinds(&priorities(&items, &[ask(1, "mine")], RULES())),
            ["ask1"]
        );
    }

    #[test]
    fn demoting_an_item_drops_it_down_the_list() {
        let items = [item("low", None, 0), item("top", None, 0)];
        let asks = [ask(1, "top"), ask(2, "low")];
        assert_eq!(kinds(&priorities(&items, &asks, RULES())), ["ask2", "ask1"]);
    }
}
