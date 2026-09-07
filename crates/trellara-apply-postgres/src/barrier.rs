pub(crate) use crate::barrier_header_context::HeaderContext;
pub(crate) use crate::barrier_header_lookup::{optional_header, required_header};
pub(crate) use crate::barrier_headers::{
    validate_chunk_header_context, validate_manifest_header_context, validate_marker_header_context,
};
pub(crate) use crate::barrier_pending::{
    same_partition_chunk, PendingBarrierTransaction, PendingChunk, PendingCommitMarker,
    PendingManifest,
};
pub(crate) use crate::barrier_pending_stats::BarrierPendingStats;
pub(crate) use crate::barrier_strict_headers::validate_strict_header_context;
use crate::ApplyDecision;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BarrierApplyStep {
    Buffered { transaction_key: String },
    Applied(ApplyStep),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyStep {
    pub transaction_id: String,
    pub commit_lsn: String,
    pub decision: ApplyDecision,
    pub applied_changes: usize,
    pub acked_messages: usize,
}
