use crate::{
    barrier_pending_blocker_codes, barrier_pending_blockers, barrier_pending_recovery_actions,
    ApplyBarrierPendingSummary,
};

impl ApplyBarrierPendingSummary {
    pub(crate) fn from_stats(stats: trellara_apply_postgres::BarrierPendingStats) -> Self {
        Self {
            transactions: stats.transactions,
            with_manifest: stats.with_manifest,
            with_commit_marker: stats.with_commit_marker,
            invalid_commit_marker: stats.invalid_commit_marker,
            missing_manifest: stats.missing_manifest,
            missing_commit_marker: stats.missing_commit_marker,
            complete_chunk_sets: stats.complete_chunk_sets,
            expected_chunks: stats.expected_chunks,
            buffered_chunks: stats.buffered_chunks,
            missing_chunks: stats.missing_chunks,
            extra_chunks: stats.extra_chunks,
            buffered_messages: stats.buffered_messages,
            blockers: barrier_pending_blockers(&stats),
            blocker_codes: barrier_pending_blocker_codes(&stats),
            recovery_actions: barrier_pending_recovery_actions(&stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barrier_pending_summary_lists_all_blockers_in_stable_order() {
        let summary =
            ApplyBarrierPendingSummary::from_stats(trellara_apply_postgres::BarrierPendingStats {
                missing_manifest: 1,
                missing_commit_marker: 2,
                invalid_commit_marker: 3,
                missing_chunks: 4,
                extra_chunks: 5,
                ..Default::default()
            });

        assert_eq!(
            summary.blockers,
            vec![
                "1 transaction(s) missing manifest",
                "2 transaction(s) missing commit marker",
                "3 transaction(s) have commit marker mismatches",
                "4 partition chunk(s) missing",
                "5 partition chunk(s) are not listed in the manifest",
            ]
        );
        assert_eq!(
            summary.blocker_codes,
            vec![
                "missing_manifest",
                "missing_commit_marker",
                "invalid_commit_marker",
                "missing_chunks",
                "extra_chunks"
            ]
        );
        assert_eq!(
            summary.recovery_actions,
            vec![
                "replay manifest topic for pending barrier transactions before target apply",
                "replay commit-marker topic for pending barrier transactions before target apply",
                "republish matching manifest and commit marker evidence before target apply",
                "replay partition chunk topics until every manifest partition is buffered",
                "quarantine or rewind unlisted partition chunks, then replay from the manifest boundary before target apply",
            ]
        );
    }
}
