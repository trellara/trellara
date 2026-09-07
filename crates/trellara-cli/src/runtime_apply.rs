use crate::{ApplyBarrierPendingSummary, ApplySummary};

impl ApplySummary {
    pub(crate) fn from_stats(stats: trellara_apply_postgres::ApplyRunStats) -> Self {
        let barrier_pending = ApplyBarrierPendingSummary::from_stats(stats.barrier_pending);
        Self {
            applied_transactions: stats.applied_transactions,
            skipped_duplicates: stats.skipped_duplicates,
            applied_changes: stats.applied_changes,
            acked_messages: stats.acked_messages,
            last_commit_lsn: stats.last_commit_lsn,
            barrier_pending_transactions: barrier_pending.transactions,
            barrier_pending_with_manifest: barrier_pending.with_manifest,
            barrier_pending_with_commit_marker: barrier_pending.with_commit_marker,
            barrier_pending_invalid_commit_marker: barrier_pending.invalid_commit_marker,
            barrier_pending_missing_manifest: barrier_pending.missing_manifest,
            barrier_pending_missing_commit_marker: barrier_pending.missing_commit_marker,
            barrier_pending_complete_chunk_sets: barrier_pending.complete_chunk_sets,
            barrier_pending_expected_chunks: barrier_pending.expected_chunks,
            barrier_pending_buffered_chunks: barrier_pending.buffered_chunks,
            barrier_pending_missing_chunks: barrier_pending.missing_chunks,
            barrier_pending_extra_chunks: barrier_pending.extra_chunks,
            barrier_pending_buffered_messages: barrier_pending.buffered_messages,
            barrier_pending,
        }
    }

    pub(crate) fn record_barrier_pending(
        &mut self,
        stats: trellara_apply_postgres::BarrierPendingStats,
    ) {
        let barrier_pending = ApplyBarrierPendingSummary::from_stats(stats);
        self.barrier_pending_transactions = barrier_pending.transactions;
        self.barrier_pending_with_manifest = barrier_pending.with_manifest;
        self.barrier_pending_with_commit_marker = barrier_pending.with_commit_marker;
        self.barrier_pending_invalid_commit_marker = barrier_pending.invalid_commit_marker;
        self.barrier_pending_missing_manifest = barrier_pending.missing_manifest;
        self.barrier_pending_missing_commit_marker = barrier_pending.missing_commit_marker;
        self.barrier_pending_complete_chunk_sets = barrier_pending.complete_chunk_sets;
        self.barrier_pending_expected_chunks = barrier_pending.expected_chunks;
        self.barrier_pending_buffered_chunks = barrier_pending.buffered_chunks;
        self.barrier_pending_missing_chunks = barrier_pending.missing_chunks;
        self.barrier_pending_extra_chunks = barrier_pending.extra_chunks;
        self.barrier_pending_buffered_messages = barrier_pending.buffered_messages;
        self.barrier_pending = barrier_pending;
    }
}
