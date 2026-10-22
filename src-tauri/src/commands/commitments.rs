use tauri::State;

use crate::app::App;
use crate::domain::commitment::{Commitment, CommitmentInput};
use crate::error::AppResult;
use crate::store::commitments;

/// Adds a meeting or appointment by hand. Calendar ones arrive through the calendar connector.
#[tauri::command]
pub async fn add_commitment(app: State<'_, App>, commitment: CommitmentInput) -> AppResult<()> {
    let c = Commitment {
        id: 0,
        area: app.settings()?.known_area(&commitment.area),
        title: commitment.title,
        starts_at: commitment.starts_at,
        ends_at: commitment.ends_at,
        location: commitment.location,
        join_url: commitment.join_url,
        source: "manual".to_owned(),
        external_id: None,
        hidden: false,
    };
    commitments::insert(&app.db, &c).await
}

#[tauri::command]
pub async fn delete_commitment(app: State<'_, App>, id: i64) -> AppResult<()> {
    commitments::delete(&app.db, id).await
}

/// Hides a meeting, e.g. a calendar event the user doesn't care about. Re-syncs keep it hidden.
#[tauri::command]
pub async fn hide_commitment(app: State<'_, App>, id: i64) -> AppResult<()> {
    let c = commitments::get(&app.db, id).await?;
    commitments::update(&app.db, &Commitment { hidden: true, ..c }).await
}
