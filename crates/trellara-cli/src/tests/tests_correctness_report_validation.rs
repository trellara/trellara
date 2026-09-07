use super::*;

#[test]
fn correctness_report_is_not_ready_with_validation_drift() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: healthy_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(1_000_000),
        source_stream_spill_threshold_changes:
            trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        source_stream_spill_dir: None,
        source: Some(caught_up_lag()),
        target: Some(caught_up_lag()),
        partition_watermarks: None,
        source_schema_drift: None,
        latest_quarantine: None,
        latest_reseed: None,
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(latest_snapshot_run()),
        latest_validation: Some(latest_validation(false)),
        recovery_actions: Vec::new(),
    }));

    assert!(!report.ready);
    assert!(!report.latest_validation_converged);
    assert_eq!(report.latest_checksum_status, ChecksumStatus::Mismatch);
    let failure = report.latest_failure.expect("latest failure");
    assert_eq!(failure.code, "validation_drift");
    assert_eq!(failure.severity, FlowAlertSeverity::Warning);
    assert_eq!(
        failure.occurred_at.as_deref(),
        Some("2026-08-11 10:10:00+00")
    );
    assert!(failure.message.contains("public.sales"));
    assert_eq!(report.issues.len(), 1);
    assert!(report.issues[0].contains("latest validation found drift"));
    assert!(report.issues[0].contains("public.sales"));
    assert!(report.issues[0].contains("public.refunds"));
    assert_eq!(
        report.recommended_actions,
        vec!["run trellara verify after repair; use trellara reseed if checksum drift remains"]
    );
}

#[test]
fn correctness_report_marks_checksum_unknown_without_validation() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: healthy_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(1_000_000),
        source_stream_spill_threshold_changes:
            trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        source_stream_spill_dir: None,
        source: Some(caught_up_lag()),
        target: Some(caught_up_lag()),
        partition_watermarks: None,
        source_schema_drift: None,
        latest_quarantine: None,
        latest_reseed: None,
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(latest_snapshot_run()),
        latest_validation: None,
        recovery_actions: Vec::new(),
    }));

    assert!(!report.ready);
    assert!(!report.latest_validation_converged);
    assert_eq!(report.latest_checksum_status, ChecksumStatus::Unknown);
    let validation_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "checksum_validation")
        .expect("checksum validation check");
    assert_eq!(
        validation_check.status,
        CorrectnessProofStatus::MissingEvidence
    );
    assert_eq!(
        validation_check.issue_code.as_deref(),
        Some("checksum_validation_missing_evidence")
    );
    assert_eq!(report.missing_evidence_proof_check_count, 1);
    assert!(report.recommended_actions.contains(
        &"run trellara verify after repair; use trellara reseed if checksum drift remains"
            .to_string()
    ));
}

#[test]
fn correctness_report_rejects_stale_converged_validation() {
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();

    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: healthy_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(1_000_000),
        source_stream_spill_threshold_changes:
            trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        source_stream_spill_dir: None,
        source: Some(caught_up_lag()),
        target: Some(caught_up_lag()),
        partition_watermarks: None,
        source_schema_drift: None,
        latest_quarantine: None,
        latest_reseed: None,
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(latest_snapshot_run()),
        latest_validation: Some(validation),
        recovery_actions: Vec::new(),
    }));

    assert!(!report.ready);
    assert!(report.latest_validation_converged);
    assert!(!report.latest_validation_current);
    assert_eq!(report.latest_checksum_status, ChecksumStatus::Match);
    let validation_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "checksum_validation")
        .expect("checksum validation check");
    assert_eq!(validation_check.status, CorrectnessProofStatus::AtRisk);
    assert!(validation_check
        .evidence
        .contains("validation watermarks are stale"));
    assert!(report.recommended_actions.contains(
        &"run trellara verify until validation watermarks reach the current source and target checkpoints"
            .to_string()
    ));
}
