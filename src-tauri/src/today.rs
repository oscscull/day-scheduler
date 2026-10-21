//! Derives what Today shows from the ranking and the calendar.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::ask::Ask;
use crate::domain::commitment::Commitment;
use crate::domain::item::WorkItem;

/// An ask, with the work item it belongs to.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    pub ask: Ask,
    pub item_id: String,
    pub item_title: String,
}

/// Asks on the highest-ranked work first, capped at `limit`.
/// `items` must already be sorted by rank.
pub fn priorities(items: &[WorkItem], asks: &[Ask], limit: usize) -> Vec<Priority> {
    items
        .iter()
        .flat_map(|item| {
            asks.iter()
                .filter(|a| a.item_id == item.id)
                .map(|a| Priority {
                    ask: a.clone(),
                    item_id: item.id.clone(),
                    item_title: item.title.clone(),
                })
        })
        .take(limit)
        .collect()
}

/// Commitments that have not ended yet, soonest first.
pub fn upcoming(mut commitments: Vec<Commitment>, now: DateTime<Utc>) -> Vec<Commitment> {
    commitments.retain(|c| c.ends_at > now);
    commitments.sort_by_key(|c| c.starts_at);
    commitments
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::domain::item::Stage;

    fn item(id: &str, rank: i64) -> WorkItem {
        WorkItem {
            id: id.to_owned(),
            title: id.to_owned(),
            state_label: String::new(),
            stage: Stage::InProgress,
            status: None,
            holder: String::new(),
            node: String::new(),
            opened_at: Utc::now(),
            rank,
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

    #[test]
    fn top_item_with_three_asks_fills_today() {
        let items = [item("top", 0), item("low", 1)];
        let asks = [ask(1, "low"), ask(2, "top"), ask(3, "top"), ask(4, "top")];
        let ids: Vec<i64> = priorities(&items, &asks, 3)
            .iter()
            .map(|p| p.ask.id)
            .collect();
        assert_eq!(ids, [2, 3, 4]);
    }

    #[test]
    fn demoting_an_item_drops_its_asks() {
        let items = [item("low", 0), item("top", 1)];
        let asks = [ask(1, "low"), ask(2, "top"), ask(3, "top"), ask(4, "top")];
        let ids: Vec<i64> = priorities(&items, &asks, 3)
            .iter()
            .map(|p| p.ask.id)
            .collect();
        assert_eq!(ids, [1, 2, 3]);
    }

    #[test]
    fn upcoming_skips_finished_meetings() {
        let now = Utc::now();
        let meeting = |id, start: i64| Commitment {
            id,
            title: String::new(),
            starts_at: now + Duration::minutes(start),
            ends_at: now + Duration::minutes(start + 30),
            location: None,
        };
        let ids: Vec<i64> = upcoming(vec![meeting(1, 60), meeting(2, -90), meeting(3, 10)], now)
            .iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, [3, 1]);
    }
}
