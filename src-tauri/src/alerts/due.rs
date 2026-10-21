//! Which meeting, if any, should have its alert up right now.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

use crate::domain::commitment::Commitment;
use crate::store::alert_state::State;

/// The window around a meeting's start when its alert shows.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub lead: Duration,
    pub linger: Duration,
}

/// The soonest meeting whose alert window is open and that isn't dealt with or snoozed.
pub fn due<'a>(
    meetings: &'a [Commitment],
    states: &HashMap<i64, State>,
    now: DateTime<Utc>,
    window: Window,
) -> Option<&'a Commitment> {
    meetings
        .iter()
        .filter(|c| c.starts_at - window.lead <= now && now <= c.starts_at + window.linger)
        .filter(|c| {
            let state = states.get(&c.id).copied().unwrap_or_default();
            !state.done && state.snoozed_until.is_none_or(|until| until <= now)
        })
        .min_by_key(|c| c.starts_at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting(id: i64, in_minutes: i64) -> Commitment {
        let start = Utc::now() + Duration::minutes(in_minutes);
        Commitment {
            id,
            title: String::new(),
            area: String::new(),
            starts_at: start,
            ends_at: start + Duration::minutes(30),
            location: None,
            join_url: None,
            source: String::new(),
            external_id: None,
            hidden: false,
        }
    }

    const W: Window = Window {
        lead: Duration::minutes(5),
        linger: Duration::minutes(3),
    };

    #[test]
    fn shows_only_inside_the_window() {
        let list = [meeting(1, 20), meeting(2, 4), meeting(3, -10)];
        assert_eq!(
            due(&list, &HashMap::new(), Utc::now(), W).map(|c| c.id),
            Some(2)
        );
    }

    #[test]
    fn done_and_snoozed_alerts_stay_away() {
        let list = [meeting(1, 2)];
        let now = Utc::now();
        let done = HashMap::from([(
            1,
            State {
                done: true,
                snoozed_until: None,
            },
        )]);
        assert!(due(&list, &done, now, W).is_none());
        let snoozed = HashMap::from([(
            1,
            State {
                done: false,
                snoozed_until: Some(now + Duration::seconds(60)),
            },
        )]);
        assert!(due(&list, &snoozed, now, W).is_none());
        assert!(due(&list, &snoozed, now + Duration::seconds(61), W).is_some());
    }
}
