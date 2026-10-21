use serde::{Deserialize, Serialize};

/// Something on a work item that needs a human: a design fork, a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ask {
    pub id: i64,
    pub item_id: String,
    pub question: String,
    pub context: String,
}
