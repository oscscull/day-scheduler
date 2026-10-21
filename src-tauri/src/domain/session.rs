use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A Claude Code session Quarterdeck can come back to. Claude only finds a session from the folder it
/// ran in, so the folder travels with the id.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionRef {
    pub id: String,
    /// The working directory the session ran in. Missing means Quarterdeck's own.
    pub cwd: Option<String>,
}
