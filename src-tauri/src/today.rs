//! Upcoming commitments for Today. Priorities live in `priorities`.

use crate::domain::commitment::Commitment;
use chrono::{DateTime, Utc};

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

    #[test]
    fn upcoming_skips_finished_meetings() {
        let now = Utc::now();
        let meeting = |id, start: i64| Commitment {
            id,
            title: String::new(),
            area: String::new(),
            starts_at: now + Duration::minutes(start),
            ends_at: now + Duration::minutes(start + 30),
            location: None,
            join_url: None,
            source: String::new(),
            external_id: None,
            hidden: false,
        };
        let ids: Vec<i64> = upcoming(vec![meeting(1, 60), meeting(2, -90), meeting(3, 10)], now)
            .iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, [3, 1]);
    }
}
