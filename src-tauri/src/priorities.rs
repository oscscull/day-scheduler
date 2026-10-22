//! What needs the user, derived from whose turn each item is and ranked by the work. Never ordered
//! by hand.
//!
//! In rank order: questions raised on an item, else the item itself when it's in flight and the
//! user's turn. Work that is the agent's or others' turn never appears; when their time runs out
//! the turn comes back here.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::ask::Ask;
use crate::domain::item::{Stage, WorkItem};
use crate::domain::turn::Turn;

/// Why something is a priority. The UI words each kind from the copy file.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reason {
    /// A question or design fork someone raised on the item.
    Ask { ask: Ask },
    /// It's the user's turn on the item, e.g. a PR to review or merge.
    Yours,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    #[serde(flatten)]
    pub reason: Reason,
    pub item_id: String,
    pub item_title: String,
    pub item_label: String,
    pub area: String,
    /// When the item reached its current state.
    pub since: DateTime<Utc>,
    /// The item's ticket or PR, so the user can act on it in one click.
    pub link: Option<String>,
}

fn priority(item: &WorkItem, reason: Reason) -> Priority {
    Priority {
        reason,
        item_id: item.id.clone(),
        item_title: item.title.clone(),
        item_label: item.state_label.clone(),
        area: item.area.clone(),
        since: item.state_since,
        link: item.link.clone(),
    }
}

const fn yours(item: &WorkItem) -> bool {
    matches!(item.turn, Turn::You) && matches!(item.stage, Stage::InProgress | Stage::InReview)
}

/// Questions on the item, or the item itself when it's the user's turn.
fn needs_you(item: &WorkItem, asks: &[Ask]) -> Vec<Priority> {
    let mine: Vec<Priority> = asks
        .iter()
        .filter(|a| a.item_id == item.id)
        .map(|a| priority(item, Reason::Ask { ask: a.clone() }))
        .collect();
    if mine.is_empty() && yours(item) {
        return vec![priority(item, Reason::Yours)];
    }
    mine
}

/// Every priority, in order. `items` must already be sorted by rank.
pub fn priorities(items: &[WorkItem], asks: &[Ask]) -> Vec<Priority> {
    items
        .iter()
        .filter(|i| i.stage != Stage::Done)
        .flat_map(|i| needs_you(i, asks))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, turn: Turn, stage: Stage) -> WorkItem {
        let now = Utc::now();
        WorkItem {
            id: id.to_owned(),
            title: id.to_owned(),
            state_label: String::new(),
            stage,
            status: None,
            turn,
            turn_until: None,
            waiting_on: None,
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

    fn ask(id: i64, item_id: &str) -> Ask {
        Ask {
            id,
            item_id: item_id.to_owned(),
            question: String::new(),
            context: String::new(),
            ..Ask::default()
        }
    }

    fn kinds(list: &[Priority]) -> Vec<String> {
        list.iter()
            .map(|p| match &p.reason {
                Reason::Ask { ask } => format!("ask{}", ask.id),
                Reason::Yours => format!("yours {}", p.item_id),
            })
            .collect()
    }

    #[test]
    fn only_the_users_turns_and_questions_in_rank_order() {
        let items = [
            item("reviewing", Turn::Others, Stage::InReview),
            item("mine", Turn::You, Stage::InReview),
            item("asked", Turn::Agent, Stage::InProgress),
            item("working", Turn::Agent, Stage::InProgress),
            item("someday", Turn::You, Stage::UpNext),
            item("ready", Turn::You, Stage::InReview),
        ];
        let asks = [ask(7, "asked")];
        assert_eq!(
            kinds(&priorities(&items, &asks)),
            ["yours mine", "ask7", "yours ready"]
        );
    }

    #[test]
    fn a_question_stands_in_for_the_item_itself() {
        let items = [item("mine", Turn::You, Stage::InReview)];
        assert_eq!(kinds(&priorities(&items, &[ask(1, "mine")])), ["ask1"]);
    }
}
