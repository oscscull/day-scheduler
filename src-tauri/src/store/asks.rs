//! Open questions for the user. Answering keeps the row only until the answer is delivered.

use super::client::{self, AskRow};
use super::convert::{int, key, text_key};
use super::db::{Db, gone_ok};
use crate::domain::ask::{Answered, Ask};
use crate::domain::session::SessionRef;
use crate::error::{AppError, AppResult};

/// An ask and its answer, if given.
type WithAnswer = (Ask, Option<String>);

fn from_row(row: AskRow) -> AppResult<WithAnswer> {
    let cwd = row.cwd;
    let ask = Ask {
        id: int(&row.id)?,
        item_id: row.item_id.into_raw(),
        question: row.question,
        context: row.context,
        options: serde_json::from_value(row.options).unwrap_or_default(),
        asker: row.session.map(|id| SessionRef { id, cwd }),
    };
    Ok((ask, row.answer))
}

fn asks(rows: Vec<AskRow>) -> AppResult<Vec<Ask>> {
    rows.into_iter()
        .map(|r| from_row(r).map(|(a, _)| a))
        .collect()
}

/// Open asks, not yet answered.
pub async fn list(db: &Db) -> AppResult<Vec<Ask>> {
    asks(db.api().open_asks(client::OpenAsksInput {}, ()).await?)
}

pub async fn for_item(db: &Db, item_id: &str) -> AppResult<Vec<Ask>> {
    let input = client::OpenAsksForInput {
        item: text_key(item_id),
    };
    asks(db.api().open_asks_for(input, ()).await?)
}

/// Answers not yet taken back to whoever asked.
pub async fn answered(db: &Db) -> AppResult<Vec<Answered>> {
    let rows = db
        .api()
        .answered_asks(client::AnsweredAsksInput {}, ())
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let (ask, Some(answer)) = from_row(row)? {
            out.push(Answered { ask, answer });
        }
    }
    Ok(out)
}

pub async fn get(db: &Db, id: i64) -> AppResult<Ask> {
    let input = client::AskInput { id: key(id) };
    let row = db.api().ask(input, ()).await?;
    let row = row.ok_or_else(|| AppError::NotFound(format!("ask {id}")))?;
    Ok(from_row(row)?.0)
}

/// Returns the new ask's id.
pub async fn insert(db: &Db, ask: &Ask) -> AppResult<i64> {
    let asker = ask.asker.as_ref();
    let input = client::AddAskInput {
        item: text_key(&ask.item_id),
        question: ask.question.clone(),
        context: ask.context.clone(),
        options: serde_json::to_value(&ask.options)?,
        session: asker.map(|s| s.id.clone()),
        cwd: asker.and_then(|s| s.cwd.clone()),
    };
    let row = db.api().add_ask(input, ()).await?;
    int(&row.id)
}

pub async fn answer(db: &Db, id: i64, answer: &str) -> AppResult<()> {
    let input = client::AnswerAskInput {
        id: key(id),
        answer: answer.to_owned(),
    };
    Ok(db.api().answer_ask(input, ()).await?)
}

/// Removes an ask: dismissed, or its answer delivered.
pub async fn remove(db: &Db, id: i64) -> AppResult<()> {
    let input = client::RemoveAskInput { id: key(id) };
    gone_ok(db.api().remove_ask(input, ()).await.map_err(AppError::from))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::item::{Stage, WorkItem};
    use crate::store::items;

    #[tokio::test]
    async fn answered_asks_leave_the_open_list_until_delivered() {
        let db = Db::in_memory().await;
        let now = Utc::now();
        let item = WorkItem {
            id: "a".to_owned(),
            title: "t".to_owned(),
            state_label: String::new(),
            stage: Stage::InProgress,
            status: None,
            holder: String::new(),
            area: "work".to_owned(),
            link: None,
            source: items::MANUAL.to_owned(),
            external_id: None,
            opened_at: now,
            state_since: now,
            rank: 0,
        };
        items::save(&db, &item).await.unwrap();
        let ask = Ask {
            item_id: "a".to_owned(),
            question: "Merge?".to_owned(),
            options: vec!["Yes".to_owned(), "No".to_owned()],
            asker: Some(SessionRef {
                id: "s1".to_owned(),
                cwd: Some("/repo".to_owned()),
            }),
            ..Ask::default()
        };
        insert(&db, &ask).await.unwrap();
        let open = list(&db).await.unwrap();
        assert_eq!(open.first().unwrap().options, ["Yes", "No"]);
        answer(&db, open.first().unwrap().id, "Yes").await.unwrap();
        assert!(list(&db).await.unwrap().is_empty());
        let done = answered(&db).await.unwrap();
        let first = done.first().unwrap();
        assert_eq!(first.answer, "Yes");
        assert_eq!(
            first.ask.asker.as_ref().unwrap().cwd.as_deref(),
            Some("/repo")
        );
    }
}
