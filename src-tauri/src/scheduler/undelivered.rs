//! An answer or message nobody can take: no Claude session holds the item and its repo has no
//! checkout on this machine. It isn't dropped quietly: the item stops claiming Claude has it, its
//! history says why, and the user gets an urgent item with what they said.

use chrono::{DateTime, Utc};

use crate::domain::ask::Answered;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Status};
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::{Db, events, items, urgent};
use crate::work;

const WHY: &str = "no Claude session holds it, and there's no checkout of its repo on this machine \
                   (tell Quarterdeck where it is with set_project)";

pub async fn report(db: &Db, answered: &Answered, now: DateTime<Utc>) -> AppResult<()> {
    let item = items::get(db, &answered.ask.item_id).await?;
    let text = format!("Not passed to Claude: {WHY}");
    events::record(db, &item.id, now, &Change::Note { text }).await?;
    let raised = Urgent {
        id: 0,
        title: format!("Claude couldn't take your message on {}", item.id),
        detail: format!("“{}”: {WHY}.", answered.answer),
        area: item.area.clone(),
        raised_at: now,
        schedule_id: None,
    };
    urgent::raise(db, &raised).await?;
    let input = ItemInput {
        id: item.id,
        title: item.title,
        state_label: "not passed to Claude".to_owned(),
        stage: item.stage,
        status: item.status.filter(|s| *s != Status::Proceeding),
        holder: "you".to_owned(),
        area: item.area,
        link: item.link,
    };
    work::save(db, input, now).await.map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ask::Ask;
    use crate::domain::item::Stage;

    #[tokio::test]
    async fn undeliverable_messages_reach_the_user_instead() {
        let db = Db::in_memory().await;
        let input = ItemInput {
            id: "B-1".to_owned(),
            title: "t".to_owned(),
            state_label: "acting on your answer".to_owned(),
            stage: Stage::InProgress,
            status: Some(Status::Proceeding),
            holder: "Claude".to_owned(),
            area: "work".to_owned(),
            link: None,
        };
        work::save(&db, input, Utc::now()).await.unwrap();
        let answered = Answered {
            ask: Ask {
                item_id: "B-1".to_owned(),
                ..Ask::default()
            },
            answer: "fix it".to_owned(),
        };
        report(&db, &answered, Utc::now()).await.unwrap();

        let item = items::get(&db, "B-1").await.unwrap();
        assert_eq!(
            (item.holder.as_str(), item.status, item.stage),
            ("you", None, Stage::InProgress)
        );
        let raised = urgent::list(&db).await.unwrap();
        assert!(raised.first().unwrap().detail.contains("fix it"));
        let history = events::for_item(&db, "B-1").await.unwrap();
        assert!(history.iter().any(|e| matches!(
            &e.change,
            Change::Note { text } if text.starts_with("Not passed to Claude")
        )));
    }
}
