//! Parking an item: the user sets it aside, so it leaves every list (Today, the board, the queue,
//! the watchdog) until unparked. Syncs keep it parked; GitHub closing its issue or PR still ends it.

use chrono::{DateTime, Utc};

use crate::domain::event::Change;
use crate::error::AppResult;
use crate::store::{Db, events, items};

/// Whether to park an item, and the user's reason if they gave one.
#[derive(Debug, Clone, Copy)]
pub struct Parking<'a> {
    pub parked: bool,
    pub why: Option<&'a str>,
}

/// Parks or unparks an item, noting it (and why, if given) in its history.
pub async fn set(db: &Db, id: &str, choice: Parking<'_>, now: DateTime<Utc>) -> AppResult<()> {
    items::get(db, id).await?;
    items::set_parked(db, id, choice.parked).await?;
    let what = if choice.parked { "Parked" } else { "Unparked" };
    let text = choice
        .why
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map_or_else(|| what.to_owned(), |w| format!("{what}: {w}"));
    events::record(db, id, now, &Change::Note { text }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::{ItemInput, Stage};
    use crate::work;

    #[tokio::test]
    async fn a_parked_item_leaves_the_lists_until_unparked() {
        let db = Db::in_memory().await;
        let input = ItemInput {
            id: "app#7".to_owned(),
            title: "Old idea".to_owned(),
            state_label: String::new(),
            stage: Stage::UpNext,
            status: None,
            holder: String::new(),
            area: "projects".to_owned(),
            link: None,
        };
        work::save(&db, input.clone(), Utc::now()).await.unwrap();
        let park = Parking {
            parked: true,
            why: Some("not now"),
        };
        set(&db, "app#7", park, Utc::now()).await.unwrap();
        assert!(items::list(&db).await.unwrap().is_empty());
        assert_eq!(items::parked(&db).await.unwrap().len(), 1);
        // An edit (or a sync) doesn't unpark it.
        work::save(&db, input, Utc::now()).await.unwrap();
        assert!(items::is_parked(&db, "app#7").await.unwrap());
        let unpark = Parking {
            parked: false,
            why: None,
        };
        set(&db, "app#7", unpark, Utc::now()).await.unwrap();
        assert_eq!(items::list(&db).await.unwrap().len(), 1);
    }
}
