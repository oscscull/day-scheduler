//! What GitHub says happened to one PR. An item is only closed on GitHub's word: a PR missing
//! from the open-PR search may just be past its page, or one GitHub couldn't describe.

use super::fetch::gh;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fate {
    Open,
    Merged,
    Closed,
}

fn parse(state: &str) -> AppResult<Fate> {
    match state {
        "OPEN" => Ok(Fate::Open),
        "MERGED" => Ok(Fate::Merged),
        "CLOSED" => Ok(Fate::Closed),
        other => Err(AppError::Connector(format!(
            "gh: unknown PR state {other:?}"
        ))),
    }
}

/// The fate of the PR with external id `owner/repo#12`.
pub async fn of(external_id: &str) -> AppResult<Fate> {
    let (repo, number) = external_id
        .split_once('#')
        .ok_or_else(|| AppError::Connector(format!("not a PR id: {external_id}")))?;
    let args = [
        "pr", "view", number, "--repo", repo, "--json", "state", "--jq", ".state",
    ];
    parse(&gh(&args).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_githubs_own_states_count() {
        assert_eq!(parse("MERGED").unwrap(), Fate::Merged);
        assert_eq!(parse("CLOSED").unwrap(), Fate::Closed);
        assert_eq!(parse("OPEN").unwrap(), Fate::Open);
        assert!(parse("").is_err());
    }
}
