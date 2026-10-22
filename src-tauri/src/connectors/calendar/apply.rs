//! Writes calendar occurrences into Quarterdeck as meetings, respecting hides and re-filing.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::parse::Occurrence;
use crate::config::calendar::Feed;
use crate::domain::commitment::Commitment;
use crate::error::AppResult;
use crate::store::{Db, commitments, sync_state};

pub fn source(feed: &Feed) -> String {
    format!("calendar:{}", feed.name)
}

/// The fields the calendar owns. Area and hidden belong to the user.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Snapshot {
    title: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    location: Option<String>,
    #[serde(default)]
    join_url: Option<String>,
}

fn snapshot(o: &Occurrence) -> Snapshot {
    Snapshot {
        title: o.title.clone(),
        start: o.start,
        end: o.end,
        location: o.location.clone(),
        join_url: o.join_url.clone(),
    }
}

async fn upsert(db: &Db, src: &str, feed: &Feed, o: &Occurrence) -> AppResult<()> {
    let new = snapshot(o);
    let old: Option<Snapshot> = sync_state::get(db, src, &o.id)
        .await?
        .and_then(|json| serde_json::from_str(&json).ok());
    match commitments::by_external(db, src, &o.id).await? {
        None => {
            commitments::insert(
                db,
                &Commitment {
                    id: 0,
                    title: new.title.clone(),
                    area: feed.area.clone(),
                    starts_at: new.start,
                    ends_at: new.end,
                    location: new.location.clone(),
                    join_url: new.join_url.clone(),
                    source: src.to_owned(),
                    external_id: Some(o.id.clone()),
                    hidden: false,
                },
            )
            .await?;
        }
        Some(c) if old.as_ref() != Some(&new) => {
            let moved = Commitment {
                title: new.title.clone(),
                starts_at: new.start,
                ends_at: new.end,
                location: new.location.clone(),
                join_url: new.join_url.clone(),
                ..c
            };
            commitments::update(db, &moved).await?;
        }
        Some(_) => {}
    }
    sync_state::set(db, src, &o.id, &serde_json::to_string(&new)?).await
}

/// Syncs one feed's occurrences. Events in the window that are gone upstream are removed.
pub async fn apply(
    db: &Db,
    feed: &Feed,
    occurrences: &[Occurrence],
    from: DateTime<Utc>,
) -> AppResult<()> {
    let src = source(feed);
    let mut seen = HashSet::new();
    for o in occurrences {
        upsert(db, &src, feed, o).await?;
        seen.insert(o.id.clone());
    }
    for c in commitments::from_source(db, &src).await? {
        let Some(ext) = c.external_id else { continue };
        if !seen.contains(&ext) && c.starts_at >= from {
            commitments::delete(db, c.id).await?;
            sync_state::forget(db, &src, &ext).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::config::calendar::Origin;

    fn occ(title: &str, start: DateTime<Utc>) -> Occurrence {
        Occurrence {
            id: "u@1".to_owned(),
            title: title.to_owned(),
            start,
            end: start + Duration::minutes(30),
            location: None,
            join_url: None,
            all_day: false,
        }
    }

    #[tokio::test]
    async fn hides_and_area_moves_survive_resyncs() {
        let db = Db::in_memory().await;
        let feed = Feed {
            name: "home".to_owned(),
            origin: Origin::Link { url: String::new() },
            area: "life".to_owned(),
            ignore_titles: vec![],
        };
        let now = Utc::now();
        apply(&db, &feed, &[occ("Dentist", now)], now)
            .await
            .unwrap();
        let mut c = commitments::from_source(&db, "calendar:home")
            .await
            .unwrap()
            .remove(0);
        c.hidden = true;
        c.area = "work".to_owned();
        commitments::update(&db, &c).await.unwrap();
        apply(&db, &feed, &[occ("Dentist (moved)", now)], now)
            .await
            .unwrap();
        let after = commitments::from_source(&db, "calendar:home")
            .await
            .unwrap()
            .remove(0);
        assert!(after.hidden && after.area == "work" && after.title == "Dentist (moved)");
        apply(&db, &feed, &[], now).await.unwrap();
        assert!(
            commitments::from_source(&db, "calendar:home")
                .await
                .unwrap()
                .is_empty()
        );
    }
}
