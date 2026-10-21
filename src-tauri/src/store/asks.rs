//! Open questions for the user. Answering keeps the row only until the answer is delivered.

use rusqlite::{Connection, Row, params};

use crate::domain::ask::{Answered, Ask};
use crate::domain::session::SessionRef;
use crate::error::AppResult;

/// An ask and its answer, if given.
type WithAnswer = (Ask, Option<String>);

const COLUMNS: &str = "id, item_id, question, context, options, session, cwd, answer";

fn from_row(row: &Row<'_>) -> rusqlite::Result<WithAnswer> {
    let options: String = row.get(4)?;
    let session: Option<String> = row.get(5)?;
    let ask = Ask {
        id: row.get(0)?,
        item_id: row.get(1)?,
        question: row.get(2)?,
        context: row.get(3)?,
        options: serde_json::from_str(&options).unwrap_or_default(),
        asker: session.map(|id| SessionRef { id, cwd: None }),
    };
    let cwd: Option<String> = row.get(6)?;
    let ask = Ask {
        asker: ask.asker.map(|s| SessionRef { cwd, ..s }),
        ..ask
    };
    Ok((ask, row.get(7)?))
}

fn query(conn: &Connection, filter: &str, arg: &str) -> AppResult<Vec<WithAnswer>> {
    let sql = format!("SELECT {COLUMNS} FROM asks WHERE {filter} ORDER BY id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([arg], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Open asks, not yet answered.
pub fn list(conn: &Connection) -> AppResult<Vec<Ask>> {
    let all = query(conn, "answer IS NULL AND ?1 = ?1", "")?;
    Ok(all.into_iter().map(|(a, _)| a).collect())
}

pub fn for_item(conn: &Connection, item_id: &str) -> AppResult<Vec<Ask>> {
    let all = query(conn, "answer IS NULL AND item_id = ?1", item_id)?;
    Ok(all.into_iter().map(|(a, _)| a).collect())
}

/// Answers not yet taken back to whoever asked.
pub fn answered(conn: &Connection) -> AppResult<Vec<Answered>> {
    let all = query(conn, "answer IS NOT NULL AND ?1 = ?1", "")?;
    Ok(all
        .into_iter()
        .filter_map(|(ask, answer)| {
            Some(Answered {
                ask,
                answer: answer?,
            })
        })
        .collect())
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Ask> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM asks WHERE id = ?1"))?;
    Ok(stmt.query_row([id], from_row)?.0)
}

/// Returns the new ask's id.
pub fn insert(conn: &Connection, ask: &Ask) -> AppResult<i64> {
    let asker = ask.asker.as_ref();
    conn.execute(
        "INSERT INTO asks (item_id, question, context, options, session, cwd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            ask.item_id,
            ask.question,
            ask.context,
            serde_json::to_string(&ask.options)?,
            asker.map(|s| s.id.as_str()),
            asker.and_then(|s| s.cwd.as_deref()),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn answer(conn: &Connection, id: i64, answer: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE asks SET answer = ?2 WHERE id = ?1",
        params![id, answer],
    )?;
    Ok(())
}

/// Removes an ask: dismissed, or its answer delivered.
pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM asks WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::schema;

    #[test]
    fn answered_asks_leave_the_open_list_until_delivered() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO items (id, title, state_label, stage, holder, area, rank, state_since, opened_at, source)
             VALUES ('a', 't', '', 'in_progress', '', 'work', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 'quarterdeck')",
            [],
        )
        .unwrap();
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
        insert(&conn, &ask).unwrap();
        let open = list(&conn).unwrap();
        assert_eq!(open.first().unwrap().options, ["Yes", "No"]);
        answer(&conn, open.first().unwrap().id, "Yes").unwrap();
        assert!(list(&conn).unwrap().is_empty());
        let done = answered(&conn).unwrap();
        let first = done.first().unwrap();
        assert_eq!(first.answer, "Yes");
        assert_eq!(
            first.ask.asker.as_ref().unwrap().cwd.as_deref(),
            Some("/repo")
        );
    }
}
