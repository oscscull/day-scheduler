//! Reads the user's open PRs with one GraphQL query through the `gh` command.

use serde::Deserialize;
use tokio::process::Command;

use crate::cli;
use crate::error::{AppError, AppResult};

const QUERY: &str = "query($q: String!) { search(query: $q, type: ISSUE, first: 50) { nodes { \
    ... on PullRequest { number title url isDraft repository { nameWithOwner } reviewDecision \
    comments { totalCount } reviews { totalCount } \
    reviewThreads(first: 100) { nodes { comments { totalCount } } } \
    commits(last: 1) { nodes { commit { statusCheckRollup { state } } } } } } } }";

/// One open PR, flattened to what Bridge needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pr {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub draft: bool,
    /// `APPROVED`, `CHANGES_REQUESTED`, `REVIEW_REQUIRED`, or none.
    pub review: Option<String>,
    /// `SUCCESS`, `FAILURE`, `ERROR`, `PENDING`, or none.
    pub ci: Option<String>,
    /// Comments, reviews and review-thread replies together.
    pub comments: u64,
}

async fn gh(args: &[&str]) -> AppResult<String> {
    let out = Command::new(cli::resolve("gh")).args(args).output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(AppError::Connector(format!(
            "gh: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}

pub async fn open_prs() -> AppResult<Vec<Pr>> {
    let login = gh(&["api", "user", "--jq", ".login"]).await?;
    let q = format!("q=is:pr is:open archived:false author:{login}");
    let json = gh(&["api", "graphql", "-f", &format!("query={QUERY}"), "-f", &q]).await?;
    let parsed: Response = serde_json::from_str(&json)?;
    Ok(parsed
        .data
        .search
        .nodes
        .into_iter()
        .filter_map(Node::into_pr)
        .collect())
}

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    search: Search,
}

#[derive(Debug, Deserialize)]
struct Search {
    nodes: Vec<Node>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Node {
    number: Option<u64>,
    title: String,
    url: String,
    is_draft: bool,
    repository: Option<Repo>,
    review_decision: Option<String>,
    comments: Count,
    reviews: Count,
    review_threads: Nodes<Thread>,
    commits: Nodes<CommitNode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Repo {
    name_with_owner: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Count {
    total_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Nodes<T> {
    nodes: Vec<T>,
}

impl<T> Default for Nodes<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Thread {
    comments: Count,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CommitNode {
    commit: Commit,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Commit {
    status_check_rollup: Option<Rollup>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Rollup {
    state: String,
}

impl Node {
    fn into_pr(self) -> Option<Pr> {
        let threads: u64 = self
            .review_threads
            .nodes
            .iter()
            .map(|t| t.comments.total_count)
            .sum();
        let ci = self
            .commits
            .nodes
            .into_iter()
            .next()
            .and_then(|c| c.commit.status_check_rollup)
            .map(|r| r.state);
        Some(Pr {
            repo: self.repository?.name_with_owner,
            number: self.number?,
            title: self.title,
            url: self.url,
            draft: self.is_draft,
            review: self.review_decision,
            ci,
            comments: self.comments.total_count + self.reviews.total_count + threads,
        })
    }
}
