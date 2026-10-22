//! The user answering an ask, or telling Claude what to do about an item. Either way it stays on
//! the item's history and waits for the scheduler to take it to Claude, but the item's state
//! changes at once: Claude has it if a session will get the answer, otherwise the answer stands.

use chrono::Utc;

use crate::config::settings::Settings;
use crate::domain::ask::Ask;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Status, WorkItem};
use crate::error::{AppError, AppResult};
use crate::store::{Db, asks, events, items, watch};
use crate::{work, workspace};

fn nonempty(text: &str) -> AppResult<&str> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::BadInput("that can't be empty".to_owned()));
    }
    Ok(text)
}

pub async fn give(db: &Db, settings: &Settings, ask_id: i64, answer: &str) -> AppResult<()> {
    let answer = nonempty(answer)?;
    let ask = asks::get(db, ask_id).await?;
    let text = format!("You answered “{answer}” to: {}", ask.question);
    events::record(db, &ask.item_id, Utc::now(), &Change::Note { text }).await?;
    asks::answer(db, ask_id, answer).await?;
    hand_over(db, settings, &ask, answer).await
}

/// A message about an item with no question behind it. Stored as an already-answered ask with no
/// question, so it reaches Claude the same way an answer does.
pub async fn tell(db: &Db, settings: &Settings, item_id: &str, message: &str) -> AppResult<()> {
    let message = nonempty(message)?;
    items::get(db, item_id).await?;
    let ask = Ask {
        item_id: item_id.to_owned(),
        ..Ask::default()
    };
    let id = asks::insert(db, &ask).await?;
    let text = format!("You told Claude: {message}");
    events::record(db, item_id, Utc::now(), &Change::Note { text }).await?;
    asks::answer(db, id, message).await?;
    hand_over(db, settings, &ask, message).await
}

/// Whether the scheduler will find a Claude session to take the answer: the one that asked, the
/// one holding the item, or a new one in the repo's project. Another machine's items are its own
/// to deliver.
async fn reaches_claude(
    db: &Db,
    settings: &Settings,
    ask: &Ask,
    item: &WorkItem,
) -> AppResult<bool> {
    if settings.node_for(&item.area) != settings.node || ask.asker.is_some() {
        return Ok(true);
    }
    if watch::all(db)
        .await?
        .get(&item.id)
        .is_some_and(|w| w.session.is_some())
    {
        return Ok(true);
    }
    Ok(workspace::for_item(settings, item)?.is_some())
}

/// Once nothing else on the item waits on the user, it stops showing as their decision.
async fn hand_over(db: &Db, settings: &Settings, ask: &Ask, answer: &str) -> AppResult<()> {
    let item = items::get(db, &ask.item_id).await?;
    let waiting = !asks::for_item(db, &item.id).await?.is_empty();
    if waiting || item.status != Some(Status::Decision) {
        return Ok(());
    }
    let (status, holder, label) = if reaches_claude(db, settings, ask, &item).await? {
        (
            Some(Status::Proceeding),
            "Claude".to_owned(),
            "acting on your answer".to_owned(),
        )
    } else {
        (None, item.holder.clone(), format!("you answered: {answer}"))
    };
    let input = ItemInput {
        id: item.id,
        title: item.title,
        state_label: label,
        stage: item.stage,
        status,
        holder,
        area: item.area,
        link: item.link,
    };
    work::save(db, input, Utc::now()).await.map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::Stage;
    use crate::domain::session::SessionRef;

    async fn decision(db: &Db, id: &str) {
        let input = ItemInput {
            id: id.to_owned(),
            title: "t".to_owned(),
            state_label: "your call".to_owned(),
            stage: Stage::InReview,
            status: Some(Status::Decision),
            holder: "you".to_owned(),
            area: "work".to_owned(),
            link: None,
        };
        work::save(db, input, Utc::now()).await.unwrap();
    }

    async fn ask(db: &Db, item: &str, asker: Option<SessionRef>) -> i64 {
        let ask = Ask {
            item_id: item.to_owned(),
            question: "Merge?".to_owned(),
            asker,
            ..Ask::default()
        };
        asks::insert(db, &ask).await.unwrap()
    }

    #[tokio::test]
    async fn answering_updates_the_item_at_once() {
        let db = Db::in_memory().await;
        let settings = Settings::default();
        decision(&db, "a").await;
        let session = SessionRef {
            id: "s".to_owned(),
            cwd: None,
        };
        let first = ask(&db, "a", Some(session)).await;
        give(&db, &settings, first, "Yes").await.unwrap();
        let a = items::get(&db, "a").await.unwrap();
        assert_eq!(
            (a.status, a.holder.as_str()),
            (Some(Status::Proceeding), "Claude")
        );

        decision(&db, "b").await;
        let second = ask(&db, "b", None).await;
        give(&db, &settings, second, "No").await.unwrap();
        let b = items::get(&db, "b").await.unwrap();
        assert_eq!(
            (b.status, b.state_label.as_str()),
            (None, "you answered: No")
        );
    }
}
