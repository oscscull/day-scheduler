//! Wakes parked work when an event it waits for happens: the turn comes back to the user (or the
//! agent, if the parking said so) without waiting for its time to run out.

use chrono::Utc;

use crate::config::settings::Settings;
use crate::domain::fired::Fired;
use crate::domain::item::WorkItem;
use crate::domain::life::Whose;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{Db, items};
use crate::turns::due::back_to;
use crate::turns::{self, Clock, Pass};

/// Who it goes back to, if `event` wakes `item`. An item's own turn changes never wake it, so
/// parking doesn't undo itself.
fn woken_to(item: &WorkItem, event: &Fired) -> Option<Turn> {
    let Whose::Later { then, wake } = &item.life.active()?.whose else {
        return None;
    };
    let own_turn = event.kind == "item.turn" && event.subject == item.id;
    (!own_turn && wake.iter().any(|t| event.matches(t))).then(|| back_to(*then, Turn::You))
}

pub async fn on_event(db: &Db, settings: &Settings, event: &Fired) -> AppResult<()> {
    for item in items::list(db).await? {
        if settings.node_for(&item.area) != settings.node {
            continue;
        }
        let Some(to) = woken_to(&item, event) else {
            continue;
        };
        let what = if event.about.is_empty() {
            format!("{} on {}", event.kind, event.subject)
        } else {
            event.about.clone()
        };
        let pass = Pass::to(to, format!("woke: {what}")).needing(format!(
            "What it was parked for happened ({what}): pick it back up?"
        ));
        let clock = Clock {
            limits: &settings.turns,
            now: Utc::now(),
        };
        turns::pass(db, &item.id, pass, clock).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::domain::life::{Life, Phase};
    use crate::domain::trigger::OnEvent;

    fn parked(id: &str) -> WorkItem {
        let wake = vec![OnEvent {
            on: "item.stage".to_owned(),
            when: BTreeMap::from([("item".to_owned(), "app#9".to_owned())]),
            cooldown: 0,
        }];
        let whose = Whose::Later { then: None, wake };
        WorkItem::sample(id, Life::active_as(Phase::InProgress, whose, None))
    }

    #[test]
    fn wakes_on_matching_events_only() {
        let item = parked("app#7");
        let done = Fired::new("item.stage", "work", "app#9", String::new()).with("item", "app#9");
        assert_eq!(woken_to(&item, &done), Some(Turn::You));
        let other = Fired::new("item.stage", "work", "app#8", String::new()).with("item", "app#8");
        assert_eq!(woken_to(&item, &other), None);
    }
}
