use super::client::{self, ActivityRow};
use super::convert::int;
use super::db::Db;
use super::time;
use crate::domain::activity::Activity;
use crate::error::AppResult;

fn from_row(row: ActivityRow) -> AppResult<Activity> {
    Ok(Activity {
        id: int(&row.id)?,
        text: row.text,
        area: row.area,
        at: time::get(&row.at)?,
        unread: row.unread,
    })
}

/// The most recent `limit` entries, newest first.
pub async fn recent(db: &Db, limit: u32) -> AppResult<Vec<Activity>> {
    let input = client::RecentActivityInput {
        limit: i64::from(limit),
    };
    let rows = db.api().recent_activity(input, ()).await?;
    rows.into_iter().map(from_row).collect()
}

/// Stores a note; its `id` is ignored and assigned by the database.
pub async fn insert(db: &Db, a: &Activity) -> AppResult<()> {
    let input = client::AddActivityInput {
        text: a.text.clone(),
        area: a.area.clone(),
        at: time::put(a.at),
        unread: a.unread,
    };
    Ok(db.api().add_activity(input, ()).await?)
}
