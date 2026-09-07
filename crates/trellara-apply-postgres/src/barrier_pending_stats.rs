use crate::barrier_pending::PendingBarrierTransaction;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BarrierPendingStats {
    pub transactions: u64,
    pub with_manifest: u64,
    pub with_commit_marker: u64,
    pub invalid_commit_marker: u64,
    pub missing_manifest: u64,
    pub missing_commit_marker: u64,
    pub complete_chunk_sets: u64,
    pub expected_chunks: u64,
    pub buffered_chunks: u64,
    pub missing_chunks: u64,
    pub extra_chunks: u64,
    pub buffered_messages: u64,
}

impl BarrierPendingStats {
    pub(crate) fn from_pending<'a>(
        pending_transactions: impl IntoIterator<Item = &'a PendingBarrierTransaction>,
    ) -> Self {
        let mut stats = Self::default();
        for pending in pending_transactions {
            stats.add_pending(pending);
        }
        stats
    }

    fn add_pending(&mut self, pending: &PendingBarrierTransaction) {
        self.transactions = self.transactions.saturating_add(1);
        self.with_manifest = self
            .with_manifest
            .saturating_add(u64::from(pending.manifest.is_some()));
        self.with_commit_marker = self
            .with_commit_marker
            .saturating_add(u64::from(pending.commit_marker.is_some()));
        self.invalid_commit_marker = self
            .invalid_commit_marker
            .saturating_add(u64::from(pending.has_invalid_commit_marker()));
        self.missing_manifest = self
            .missing_manifest
            .saturating_add(u64::from(pending.manifest.is_none()));
        self.missing_commit_marker = self
            .missing_commit_marker
            .saturating_add(u64::from(pending.commit_marker.is_none()));
        self.complete_chunk_sets = self
            .complete_chunk_sets
            .saturating_add(u64::from(pending.has_complete_chunk_set()));
        self.expected_chunks = self
            .expected_chunks
            .saturating_add(pending.expected_chunk_count());
        self.buffered_chunks = self
            .buffered_chunks
            .saturating_add(pending.buffered_chunk_count());
        self.missing_chunks = self
            .missing_chunks
            .saturating_add(pending.missing_chunk_count());
        self.extra_chunks = self
            .extra_chunks
            .saturating_add(pending.extra_chunk_count());
        self.buffered_messages = self
            .buffered_messages
            .saturating_add(pending.buffered_message_count());
    }
}

#[cfg(test)]
#[path = "tests/tests_barrier_pending_stats.rs"]
mod tests;
