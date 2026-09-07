use serde::Serialize;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct ApplySummary {
    pub(crate) applied_transactions: u64,
    pub(crate) skipped_duplicates: u64,
    pub(crate) applied_changes: u64,
    pub(crate) acked_messages: u64,
    pub(crate) last_commit_lsn: Option<String>,
    pub(crate) barrier_pending: ApplyBarrierPendingSummary,
    pub(crate) barrier_pending_transactions: u64,
    pub(crate) barrier_pending_with_manifest: u64,
    pub(crate) barrier_pending_with_commit_marker: u64,
    pub(crate) barrier_pending_invalid_commit_marker: u64,
    pub(crate) barrier_pending_missing_manifest: u64,
    pub(crate) barrier_pending_missing_commit_marker: u64,
    pub(crate) barrier_pending_complete_chunk_sets: u64,
    pub(crate) barrier_pending_expected_chunks: u64,
    pub(crate) barrier_pending_buffered_chunks: u64,
    pub(crate) barrier_pending_missing_chunks: u64,
    pub(crate) barrier_pending_extra_chunks: u64,
    pub(crate) barrier_pending_buffered_messages: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct ApplyBarrierPendingSummary {
    pub(crate) transactions: u64,
    pub(crate) with_manifest: u64,
    pub(crate) with_commit_marker: u64,
    pub(crate) invalid_commit_marker: u64,
    pub(crate) missing_manifest: u64,
    pub(crate) missing_commit_marker: u64,
    pub(crate) complete_chunk_sets: u64,
    pub(crate) expected_chunks: u64,
    pub(crate) buffered_chunks: u64,
    pub(crate) missing_chunks: u64,
    pub(crate) extra_chunks: u64,
    pub(crate) buffered_messages: u64,
    pub(crate) blockers: Vec<String>,
    pub(crate) blocker_codes: Vec<String>,
    pub(crate) recovery_actions: Vec<String>,
}
