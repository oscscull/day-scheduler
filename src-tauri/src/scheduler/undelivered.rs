//! An answer or message nobody can take: no agent session holds the item and its repo has no
//! checkout on this machine. It isn't dropped quietly: the turn comes back to the user with why,
//! and they get an urgent item with what they said.

use crate::domain::ask::Answered;
use crate::domain::turn::Turn;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::{Db, items, urgent};
use crate::turns::{self, Clock, Pass};

const WHY: &str = "no agent session holds it, and there's no checkout of its repo on this machine \
                   (tell Quarterdeck where it is with set_project)";

pub async fn report(db: &Db, answered: &Answered, clock: Clock<'_>) -> AppResult<()> {
    let now = clock.now;
    let item = items::get(db, &answered.ask.item_id).await?;
    let why = format!("not passed to the agent: {WHY}");
    turns::pass(db, &item.id, Pass::to(Turn::You, why), clock).await?;
    let raised = Urgent {
        id: 0,
        title: format!("Your agent couldn't take your message on {}", item.id),
        detail: format!("“{}”: {WHY}.", answered.answer),
        area: item.area.clone(),
        raised_at: now,
        schedule_id: None,
        run_id: None,
    };
    urgent::raise(db, &raised).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::config::turns::TurnSettings;
    use crate::domain::ask::Ask;
    use crate::domain::event::Change;
    use crate::domain::item::{ItemInput, Stage};
    use crate::store::events;
    use crate::work;

    #[tokio::test]
    async fn undeliverable_messages_reach_the_user_instead() {
        let db = Db::in_memory().await;
        let input = ItemInput {
            id: "B-1".to_owned(),
            title: "t".to_owned(),
            state_label: "acting on your answer".to_owned(),
            stage: Stage::InProgress,
            status: None,
            area: "work".to_owned(),
            link: None,
        };
        work::save(&db, input, Utc::now()).await.unwrap();
        let limits = TurnSettings::default();
        turns::pass(
            &db,
            "B-1",
            Pass::to(Turn::Agent, "you said: fix it"),
            Clock::now(&limits),
        )
        .await
        .unwrap();
        let answered = Answered {
            ask: Ask {
                item_id: "B-1".to_owned(),
                ..Ask::default()
            },
            answer: "fix it".to_owned(),
        };
        report(&db, &answered, Clock::now(&limits)).await.unwrap();

        assert!(items::get(&db, "B-1").await.unwrap().is_turn(Turn::You));
        let raised = urgent::list(&db).await.unwrap();
        assert!(raised.first().unwrap().detail.contains("fix it"));
        let history = events::for_item(&db, "B-1").await.unwrap();
        assert!(history.iter().any(|e| matches!(
            &e.change,
            Change::Turn { why, .. } if why.starts_with("not passed to the agent")
        )));
    }
}
