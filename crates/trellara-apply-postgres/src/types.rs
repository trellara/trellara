use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use trellara_protocol::{RelationId, TransactionEnvelope};

use crate::Result;

#[async_trait]
pub trait EnvelopeApplier: Send {
    async fn apply_envelope(&mut self, envelope: &TransactionEnvelope) -> Result<ApplyOutcome>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresApplyConfig {
    pub connection_uri: String,
    pub ensure_checkpoint_schema: bool,
    pub table_policies: Vec<ApplyTablePolicy>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplyTablePolicy {
    pub relation: RelationId,
    pub target_owned_columns: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyOutcome {
    pub decision: ApplyDecision,
    pub applied_changes: usize,
    pub commit_lsn: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyDecision {
    Applied,
    SkippedDuplicate,
}
