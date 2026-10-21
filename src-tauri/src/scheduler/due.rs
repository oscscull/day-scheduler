//! Works out which cron occurrences between two scheduler ticks to run, and which were missed.

use std::str::FromStr as _;

use chrono::{DateTime, Duration, Local, Utc};
use croner::Cron;

use crate::error::AppResult;

/// What to do about one schedule since the last tick.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// Run it now: an occurrence fell within the grace period.
    pub run: bool,
    /// The latest occurrence that is too old to run late, if any.
    pub missed: Option<DateTime<Utc>>,
    /// How many occurrences were missed in total.
    pub missed_count: usize,
}

/// Occurrences in `(since, now]`. Those older than `grace` are missed; anything newer runs once.
pub fn plan(
    expr: &str,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
    grace: Duration,
) -> AppResult<Plan> {
    let cron = Cron::from_str(expr)?;
    let mut plan = Plan::default();
    for at in cron.iter_after(since.with_timezone(&Local)) {
        let at = at.with_timezone(&Utc);
        if at > now {
            break;
        }
        if now - at > grace {
            plan.missed = Some(at);
            plan.missed_count += 1;
        } else {
            plan.run = true;
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Local
            .with_ymd_and_hms(2026, 10, 21, h, m, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    const GRACE: Duration = Duration::minutes(10);

    #[test]
    fn runs_what_just_came_due() {
        let p = plan("0 9 * * *", at(8, 59), at(9, 0), GRACE).unwrap();
        assert_eq!(
            p,
            Plan {
                run: true,
                missed: None,
                missed_count: 0
            }
        );
    }

    #[test]
    fn nothing_due_means_nothing_to_do() {
        assert_eq!(
            plan("0 9 * * *", at(9, 1), at(9, 2), GRACE).unwrap(),
            Plan::default()
        );
    }

    #[test]
    fn a_long_sleep_misses_instead_of_running_late() {
        let p = plan("0 * * * *", at(6, 30), at(9, 30), GRACE).unwrap();
        assert!(!p.run);
        assert_eq!(p.missed, Some(at(9, 0)));
        assert_eq!(p.missed_count, 3);
    }
}
