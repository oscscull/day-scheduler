use tauri::State;

use crate::app::App;
use crate::domain::commitment::{Commitment, CommitmentInput};
use crate::error::AppResult;
use crate::store::commitments;

/// Adds a meeting or appointment by hand. Calendar ones arrive through the calendar connector.
#[tauri::command]
pub fn add_commitment(app: State<'_, App>, commitment: CommitmentInput) -> AppResult<()> {
    let c = Commitment {
        id: 0,
        area: app.settings()?.known_area(&commitment.area),
        title: commitment.title,
        starts_at: commitment.starts_at,
        ends_at: commitment.ends_at,
        location: commitment.location,
    };
    app.db.with(|conn| commitments::insert(conn, &c))
}

#[tauri::command]
pub fn delete_commitment(app: State<'_, App>, id: i64) -> AppResult<()> {
    app.db.with(|conn| commitments::delete(conn, id))
}
