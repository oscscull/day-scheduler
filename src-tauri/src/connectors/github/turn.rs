//! Whose turn a PR is, by what GitHub says. GitHub never takes the turn from the agent.

use super::snapshot::Owned;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{Db, items};
use crate::turns::{self, Clock, Pass};

/// The user's turn when it waits on them (or on a CI fix nobody started), else others', waiting
/// on whoever GitHub says: reviewers or CI.
fn pass_for(owned: &Owned) -> Pass {
    match owned.holder.as_str() {
        "you" | "Claude" => Pass::to(Turn::You, owned.label.clone()),
        who => Pass::to(Turn::Others, owned.label.clone()).waiting_on(who),
    }
}

pub async fn follow(db: &Db, item_id: &str, owned: &Owned, clock: Clock<'_>) -> AppResult<()> {
    let item = items::get(db, item_id).await?;
    let pass = pass_for(owned);
    let same = (item.turn, &item.waiting_on) == (pass.to, &pass.waiting_on);
    if item.turn == Turn::Agent || same {
        return Ok(());
    }
    turns::pass(db, item_id, pass, clock).await
}
