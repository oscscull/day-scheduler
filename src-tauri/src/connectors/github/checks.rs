//! A PR's CI state from its individual checks, leaving out checks a repo treats as informational
//! (e.g. a visual-diff report). Their results become notes on the item instead.

/// How one check stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Pending,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub outcome: Outcome,
    /// What the check said about itself, e.g. "2 changed, 3 new, 0 deleted".
    pub detail: String,
}

/// A check run (GitHub Actions and apps) by its status and conclusion.
pub fn run_outcome(status: &str, conclusion: Option<&str>) -> Outcome {
    if status != "COMPLETED" {
        return Outcome::Pending;
    }
    match conclusion {
        Some("FAILURE" | "TIMED_OUT" | "CANCELLED" | "ACTION_REQUIRED" | "STARTUP_FAILURE") => {
            Outcome::Fail
        }
        _ => Outcome::Pass,
    }
}

/// A commit status (posted through the statuses API) by its state.
pub fn status_outcome(state: &str) -> Outcome {
    match state {
        "FAILURE" | "ERROR" => Outcome::Fail,
        "PENDING" | "EXPECTED" => Outcome::Pending,
        _ => Outcome::Pass,
    }
}

fn informational(check: &Check, names: &[String]) -> bool {
    names.iter().any(|n| n.eq_ignore_ascii_case(&check.name))
}

/// The rollup state (`FAILURE`, `PENDING`, `SUCCESS`) over the checks that count, or `None` when
/// there are none.
pub fn rollup(checks: &[Check], names: &[String]) -> Option<String> {
    let counted: Vec<Outcome> = checks
        .iter()
        .filter(|c| !informational(c, names))
        .map(|c| c.outcome)
        .collect();
    let state = if counted.contains(&Outcome::Fail) {
        "FAILURE"
    } else if counted.contains(&Outcome::Pending) {
        "PENDING"
    } else if counted.is_empty() {
        return None;
    } else {
        "SUCCESS"
    };
    Some(state.to_owned())
}

/// Results of informational checks worth a note: the ones that failed, e.g. "reg: 2 changed".
pub fn notes(checks: &[Check], names: &[String]) -> Vec<String> {
    checks
        .iter()
        .filter(|c| informational(c, names) && c.outcome == Outcome::Fail)
        .map(|c| {
            if c.detail.is_empty() {
                format!("{} failed", c.name)
            } else {
                format!("{}: {}", c.name, c.detail)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(name: &str, outcome: Outcome) -> Check {
        Check {
            name: name.to_owned(),
            outcome,
            detail: String::new(),
        }
    }

    #[test]
    fn informational_failures_become_notes_not_ci_failures() {
        let reg = Check {
            detail: "2 changed, 3 new".to_owned(),
            ..check("reg", Outcome::Fail)
        };
        let checks = [check("test", Outcome::Pass), reg];
        let names = ["reg".to_owned()];
        assert_eq!(rollup(&checks, &names).as_deref(), Some("SUCCESS"));
        assert_eq!(rollup(&checks, &[]).as_deref(), Some("FAILURE"));
        assert_eq!(notes(&checks, &names), ["reg: 2 changed, 3 new"]);
    }

    #[test]
    fn pending_counts_unless_something_failed() {
        let checks = [
            check("test", Outcome::Pending),
            check("lint", Outcome::Pass),
        ];
        assert_eq!(rollup(&checks, &[]).as_deref(), Some("PENDING"));
        assert_eq!(run_outcome("COMPLETED", Some("SKIPPED")), Outcome::Pass);
        assert_eq!(status_outcome("ERROR"), Outcome::Fail);
    }
}
