use super::*;

mod fixtures;
mod target_progress;
use fixtures::*;

#[test]
fn correctness_report_is_ready_when_status_evidence_is_clean() {
    let report =
        CorrectnessReportSummary::from_status(correctness_status("strict_transaction_order"));

    assert!(report.ready);
    assert_eq!(report.proof_check_count, 10);
    assert_eq!(report.verified_proof_check_count, 10);
    assert_eq!(report.at_risk_proof_check_count, 0);
    assert_eq!(report.missing_evidence_proof_check_count, 0);
    assert!(report
        .proof_checks
        .iter()
        .all(|check| check.status == CorrectnessProofStatus::Verified));
    assert!(report
        .proof_checks
        .iter()
        .all(|check| check.issue_code.is_none()));
    assert!(report
        .proof_checks
        .iter()
        .any(|check| check.code == "source_subscription_conflicts"));
    assert!(report
        .proof_checks
        .iter()
        .any(|check| check.code == "transaction_boundary"));
    assert!(report
        .proof_checks
        .iter()
        .any(|check| check.code == "snapshot_handoff"));
    assert!(report
        .proof_checks
        .iter()
        .any(|check| check.code == "source_schema_contract"));
    assert!(report.source_slot_safe);
    assert!(report.source_schema_contract_safe);
    assert!(report.source_wal_retention_safe);
    assert!(report.source_checkpoint_durable);
    assert!(report.target_caught_up);
    assert!(report.partition_watermark_ready);
    assert!(report.no_target_quarantine);
    assert!(report.latest_validation_converged);
    assert_eq!(report.latest_checksum_status, ChecksumStatus::Match);
    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    assert!(report.transaction_boundary.source_checkpoint_durable);
    assert!(report.transaction_boundary.target_checkpoint_caught_up);
    assert!(!report.transaction_boundary.manifest_barrier_required);
    assert!(report
        .transaction_boundary
        .evidence
        .contains("source_checkpoint last_seen_lsn=0/16B6C50"));
    assert!(report
        .transaction_boundary
        .evidence
        .contains("target_checkpoint last_durable_lsn=0/16B6C50"));
    assert!(report
        .transaction_boundary
        .evidence
        .contains("target_quarantine none"));
    assert!(report
        .transaction_boundary
        .visibility_contract
        .contains("one globally visible ordered envelope"));
    assert_eq!(
        report.latest_source_watermark_lsn,
        Some("0/16B6C50".to_string())
    );
    assert_eq!(
        report.latest_reseed_watermark_lsn,
        Some("0/16B8000".to_string())
    );
    assert_eq!(
        report.latest_snapshot_handoff_watermark_lsn,
        Some("0/16B8000".to_string())
    );
    assert_eq!(
        report.latest_snapshot_handoff_relation,
        Some("public.sales".to_string())
    );
    assert_eq!(
        report.latest_snapshot_run_id,
        Some("snapshot-run-1".to_string())
    );
    assert_eq!(report.latest_snapshot_state, Some("streaming".to_string()));
    assert_eq!(
        report.latest_snapshot_consistent_lsn,
        Some("0/16B8000".to_string())
    );
    assert_eq!(report.latest_failure, None);
    assert!(report.issues.is_empty());
    assert!(report.recommended_actions.is_empty());
}

#[test]
fn correctness_report_surfaces_strict_chunk_manifest_barrier() {
    let report = CorrectnessReportSummary::from_status(correctness_status(
        "strict_chunked_transaction_order",
    ));

    assert!(report.ready);
    assert_eq!(report.mode, "strict_chunked_transaction_order");
    assert_eq!(report.proof_check_count, 11);
    assert_eq!(report.verified_proof_check_count, 11);
    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    assert!(report.transaction_boundary.manifest_barrier_required);
    assert_eq!(
        report.transaction_boundary.manifest_barrier_complete,
        Some(true)
    );
    assert_eq!(
        report
            .transaction_boundary
            .global_partition_watermark_caught_up,
        None
    );
    assert!(report
        .transaction_boundary
        .guarantee
        .contains("strict chunk manifest and commit marker barrier"));
    assert!(report
        .transaction_boundary
        .visibility_contract
        .contains("every strict chunk before visibility"));
    let strict_chunk = report
        .proof_checks
        .iter()
        .find(|check| check.code == "strict_chunk_manifest_barrier")
        .expect("strict chunk proof");
    assert_eq!(strict_chunk.status, CorrectnessProofStatus::Verified);
    assert!(strict_chunk
        .evidence
        .contains("strict chunk manifest and commit marker"));
    let boundary = report
        .proof_checks
        .iter()
        .find(|check| check.code == "transaction_boundary")
        .expect("transaction boundary proof");
    assert!(boundary
        .evidence
        .contains("manifest_barrier required=true complete=true"));
    assert!(boundary
        .evidence
        .contains("strict-chunk consumers wait for the manifest"));
}
