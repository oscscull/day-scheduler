use serde::Serialize;

use crate::domain::activity::Activity;
use crate::domain::commitment::Commitment;
use crate::domain::item::Stage;
use crate::domain::todo::Todo;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::priorities::{Priority, Rules, priorities};
use crate::store::{Db, activity, asks, commitments, items, runs, todos, urgent};
use crate::today::upcoming;

const ACTIVITY_SHOWN: u32 = 12;
/// Enough upcoming commitments for any one area to still show its next two.
const UPCOMING_SHOWN: usize = 12;

/// Everything the Today screen shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub urgent: Vec<Urgent>,
    pub upcoming: Vec<Commitment>,
    /// Every ask in rank order; the UI shows the first `priority_limit` in the chosen area.
    pub priorities: Vec<Priority>,
    pub priority_limit: usize,
    pub todos: Vec<Todo>,
    pub activity: Vec<Activity>,
    pub running: usize,
    pub in_flight: usize,
}

/// What Today shows. `rules` decide which waiting items are worth a nudge.
pub async fn build(db: &Db, priority_limit: usize, rules: Rules) -> AppResult<TodayView> {
    let now = rules.now;
    let ranked = items::list(db).await?;
    let in_flight = ranked
        .iter()
        .filter(|i| !matches!(i.stage, Stage::UpNext | Stage::Done))
        .count();
    Ok(TodayView {
        urgent: urgent::list(db).await?,
        upcoming: upcoming(commitments::list(db).await?, now)
            .into_iter()
            .take(UPCOMING_SHOWN)
            .collect(),
        priorities: priorities(&ranked, &asks::list(db).await?, rules),
        priority_limit,
        todos: todos::list(db).await?,
        activity: activity::recent(db, ACTIVITY_SHOWN).await?,
        running: runs::live(db).await?.len(),
        in_flight,
    })
}
