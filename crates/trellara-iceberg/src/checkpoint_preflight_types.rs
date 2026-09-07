use serde::{Deserialize, Serialize};

use crate::IcebergTableIdentifier;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergPreflightAction {
    RecordIntentThenCommit,
    CommitAfterRecordedIntent,
    SkipAlreadyCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergPreflightDecision {
    pub target: IcebergTableIdentifier,
    pub table_commit_id: String,
    pub action: IcebergPreflightAction,
    pub reason: String,
}
