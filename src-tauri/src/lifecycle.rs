//! How setting an item's stage moves it through its life. Starting or reopening work makes it the
//! user's turn (until someone takes it on), finishing it ends the turn, and started work never goes
//! back to the backlog: it's parked or closed instead.

use crate::domain::item::Stage;
use crate::domain::life::{Active, Life, Phase, Whose};
use crate::error::{AppError, AppResult};

const STARTED: &str = "Started: what happens next?";
const REOPENED: &str = "Reopened: what happens next?";
const BACK_TO_BACKLOG: &str = "started work can't go back to up next: park it until something \
                               happens, or mark it done";

/// The user's turn on newly active work, needing `label` if it says anything, else `fallback`.
fn yours(phase: Phase, label: &str, fallback: &str) -> Life {
    let need = if label.trim().is_empty() {
        fallback
    } else {
        label
    };
    let whose = Whose::You {
        need: need.trim().to_owned(),
    };
    Life::active_as(phase, whose, None)
}

/// The life of new work at `stage`.
pub fn new(stage: Stage, label: &str) -> Life {
    match (stage, Phase::of(stage)) {
        (_, Some(phase)) => yours(phase, label, STARTED),
        (Stage::Done, None) => Life::Done,
        (Stage::UpNext | Stage::InProgress | Stage::InReview, None) => Life::Unstarted,
    }
}

/// `old` moved to `stage`. Active work keeps its turn between in progress and in review.
pub fn moved(old: &Life, stage: Stage, label: &str) -> AppResult<Life> {
    Ok(match (old, stage, Phase::of(stage)) {
        (_, Stage::Done, _) => Life::Done,
        (Life::Active(a), _, Some(phase)) => Life::Active(Active { phase, ..a.clone() }),
        (Life::Active(_), _, None) => return Err(AppError::BadInput(BACK_TO_BACKLOG.to_owned())),
        (Life::Unstarted, _, Some(phase)) => yours(phase, label, STARTED),
        (Life::Done, _, Some(phase)) => yours(phase, label, REOPENED),
        (Life::Unstarted | Life::Done, _, None) => Life::Unstarted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::turn::Turn;

    fn agents() -> Life {
        Life::active_as(Phase::InProgress, Whose::Agent, None)
    }

    #[test]
    fn started_work_never_returns_to_the_backlog() {
        assert!(moved(&agents(), Stage::UpNext, "").is_err());
        let review = moved(&agents(), Stage::InReview, "").unwrap();
        assert_eq!(
            (review.stage(), review.turn()),
            (Stage::InReview, Some(Turn::Agent))
        );
        assert_eq!(moved(&agents(), Stage::Done, "").unwrap(), Life::Done);
    }

    #[test]
    fn starting_or_reopening_work_says_what_it_needs() {
        let started = moved(&Life::Unstarted, Stage::InProgress, "").unwrap();
        assert!(matches!(
            started.whose(),
            Some(Whose::You { need: wanted }) if wanted == STARTED
        ));
        let reopened = moved(&Life::Done, Stage::InReview, "review it").unwrap();
        assert!(matches!(
            reopened.whose(),
            Some(Whose::You { need: wanted }) if wanted == "review it"
        ));
        assert_eq!(
            moved(&Life::Done, Stage::UpNext, "").unwrap(),
            Life::Unstarted
        );
        assert_eq!(new(Stage::UpNext, "x"), Life::Unstarted);
    }
}
