//! Writes calendar occurrences into Meatpuppet as meetings, respecting hides and re-filing.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::parse::Occurrence;
use crate::config::calendar::Feed;
use crate::domain::commitment::Commitment;
use crate::error::AppResult;
use crate::store::{commitments, sync_state};

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

fn upsert(conn: &Connection, src: &str, feed: &Feed, o: &Occurrence) -> AppResult<()> {
    let new = snapshot(o);
    let old: Option<Snapshot> =
        sync_state::get(conn, src, &o.id)?.and_then(|json| serde_json::from_str(&json).ok());
    match commitments::by_external(conn, src, &o.id)? {
        None => commitments::insert(
            conn,
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
        )?,
        Some(c) if old.as_ref() != Some(&new) => commitments::update(
            conn,
            &Commitment {
                title: new.title.clone(),
                starts_at: new.start,
                ends_at: new.end,
                location: new.location.clone(),
                join_url: new.join_url.clone(),
                ..c
            },
        )?,
        Some(_) => {}
    }
    sync_state::set(conn, src, &o.id, &serde_json::to_string(&new)?)
}

/// Syncs one feed's occurrences. Events in the window that are gone upstream are removed.
pub fn apply(
    conn: &Connection,
    feed: &Feed,
    occurrences: &[Occurrence],
    from: DateTime<Utc>,
) -> AppResult<()> {
    let src = source(feed);
    let mut seen = HashSet::new();
    for o in occurrences {
        upsert(conn, &src, feed, o)?;
        seen.insert(o.id.clone());
    }
    for c in commitments::from_source(conn, &src)? {
        let Some(ext) = c.external_id else { continue };
        if !seen.contains(&ext) && c.starts_at >= from {
            commitments::delete(conn, c.id)?;
            sync_state::forget(conn, &src, &ext)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::store::schema;

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

    #[test]
    fn hides_and_area_moves_survive_resyncs() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        let feed = Feed {
            name: "home".to_owned(),
            url: String::new(),
            area: "life".to_owned(),
            ignore_titles: vec![],
        };
        let now = Utc::now();
        apply(&conn, &feed, &[occ("Dentist", now)], now).unwrap();
        let mut c = commitments::from_source(&conn, "calendar:home")
            .unwrap()
            .remove(0);
        c.hidden = true;
        c.area = "work".to_owned();
        commitments::update(&conn, &c).unwrap();
        apply(&conn, &feed, &[occ("Dentist (moved)", now)], now).unwrap();
        let after = commitments::from_source(&conn, "calendar:home")
            .unwrap()
            .remove(0);
        assert!(after.hidden && after.area == "work" && after.title == "Dentist (moved)");
        apply(&conn, &feed, &[], now).unwrap();
        assert!(
            commitments::from_source(&conn, "calendar:home")
                .unwrap()
                .is_empty()
        );
    }
}
