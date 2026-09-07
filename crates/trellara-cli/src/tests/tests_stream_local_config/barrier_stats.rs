use super::*;

#[test]
fn apply_summary_exposes_pending_barrier_stats() {
    let summary = ApplySummary::from_stats(trellara_apply_postgres::ApplyRunStats {
        applied_transactions: 0,
        skipped_duplicates: 0,
        applied_changes: 0,
        acked_messages: 0,
        last_commit_lsn: None,
        barrier_pending: trellara_apply_postgres::BarrierPendingStats {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 0,
            invalid_commit_marker: 0,
            missing_manifest: 0,
            missing_commit_marker: 1,
            complete_chunk_sets: 1,
            expected_chunks: 1,
            buffered_chunks: 1,
            missing_chunks: 0,
            extra_chunks: 0,
            buffered_messages: 2,
        },
    });

    assert_eq!(summary.barrier_pending_transactions, 1);
    assert_eq!(summary.barrier_pending_with_manifest, 1);
    assert_eq!(summary.barrier_pending_with_commit_marker, 0);
    assert_eq!(summary.barrier_pending_invalid_commit_marker, 0);
    assert_eq!(summary.barrier_pending_missing_manifest, 0);
    assert_eq!(summary.barrier_pending_missing_commit_marker, 1);
    assert_eq!(summary.barrier_pending_complete_chunk_sets, 1);
    assert_eq!(summary.barrier_pending_expected_chunks, 1);
    assert_eq!(summary.barrier_pending_buffered_chunks, 1);
    assert_eq!(summary.barrier_pending_missing_chunks, 0);
    assert_eq!(summary.barrier_pending_extra_chunks, 0);
    assert_eq!(summary.barrier_pending_buffered_messages, 2);
    assert_eq!(summary.barrier_pending.transactions, 1);
    assert_eq!(summary.barrier_pending.with_manifest, 1);
    assert_eq!(summary.barrier_pending.with_commit_marker, 0);
    assert_eq!(summary.barrier_pending.invalid_commit_marker, 0);
    assert_eq!(summary.barrier_pending.missing_manifest, 0);
    assert_eq!(summary.barrier_pending.missing_commit_marker, 1);
    assert_eq!(summary.barrier_pending.complete_chunk_sets, 1);
    assert_eq!(summary.barrier_pending.expected_chunks, 1);
    assert_eq!(summary.barrier_pending.buffered_chunks, 1);
    assert_eq!(summary.barrier_pending.missing_chunks, 0);
    assert_eq!(summary.barrier_pending.extra_chunks, 0);
    assert_eq!(summary.barrier_pending.buffered_messages, 2);
    assert_eq!(
        summary.barrier_pending.blockers,
        vec!["1 transaction(s) missing commit marker"]
    );
    assert_eq!(
        summary.barrier_pending.blocker_codes,
        vec!["missing_commit_marker"]
    );
    assert_eq!(
        summary.barrier_pending.recovery_actions,
        vec!["replay commit-marker topic for pending barrier transactions before target apply"]
    );
}

#[test]
fn apply_summary_exposes_invalid_commit_marker_blocker() {
    let summary =
        ApplyBarrierPendingSummary::from_stats(trellara_apply_postgres::BarrierPendingStats {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 1,
            invalid_commit_marker: 1,
            ..Default::default()
        });

    assert_eq!(summary.invalid_commit_marker, 1);
    assert_eq!(
        summary.blockers,
        vec!["1 transaction(s) have commit marker mismatches"]
    );
    assert_eq!(summary.blocker_codes, vec!["invalid_commit_marker"]);
    assert_eq!(
        summary.recovery_actions,
        vec!["republish matching manifest and commit marker evidence before target apply"]
    );
}
