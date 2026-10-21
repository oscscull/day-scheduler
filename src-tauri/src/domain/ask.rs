use serde::{Deserialize, Serialize};

use super::session::SessionRef;

/// Something on a work item that needs a human: a design fork, a question.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ask {
    pub id: i64,
    pub item_id: String,
    pub question: String,
    pub context: String,
    /// Choices to answer with, e.g. "Yes" and "No". Empty means a written answer. The user can
    /// always write one instead.
    pub options: Vec<String>,
    /// The session that asked, so the answer can be taken back to it.
    #[serde(skip)]
    pub asker: Option<SessionRef>,
}

/// An answer waiting to be taken back to whoever asked.
#[derive(Debug, Clone)]
pub struct Answered {
    pub ask: Ask,
    pub answer: String,
}
