use super::*;

#[test]
fn correctness_report_requires_snapshot_handoff_evidence() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: None,
        latest_snapshot_run: None,
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.proof_check_count, 10);
    assert_eq!(report.missing_evidence_proof_check_count, 1);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(
        snapshot_check.status,
        CorrectnessProofStatus::MissingEvidence
    );
    assert!(report.recommended_actions.contains(
        &"run trellara snapshot --config <flow> before starting relay/apply for an initial copy"
            .to_string()
    ));
}

#[test]
fn correctness_report_blocks_partial_snapshot_handoff_until_run_is_ready() {
    let mut run = latest_snapshot_run();
    run.state = SnapshotRunState::CopyingTable;

    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(run),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.at_risk_proof_check_count, 1);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(snapshot_check.status, CorrectnessProofStatus::AtRisk);
    assert!(snapshot_check.evidence.contains("still in copying_table"));
    assert!(snapshot_check
        .evidence
        .contains("latest table handoff for public.sales"));
    assert_eq!(
        snapshot_check.recommendation.as_deref(),
        Some("resume trellara snapshot --config <flow> until the run reaches stream_handoff_ready")
    );
    assert!(report.recommended_actions.contains(
        &"resume trellara snapshot --config <flow> until the run reaches stream_handoff_ready"
            .to_string()
    ));
}

#[test]
fn correctness_report_waits_for_stream_replay_after_snapshot_handoff_ready() {
    let mut run = latest_snapshot_run();
    run.state = SnapshotRunState::StreamHandoffReady;

    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(run),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.at_risk_proof_check_count, 1);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(snapshot_check.status, CorrectnessProofStatus::AtRisk);
    assert!(snapshot_check
        .evidence
        .contains("stream replay has not yet been observed"));
    assert_eq!(
        snapshot_check.recommendation.as_deref(),
        Some(
            "start trellara relay/apply or trellara run so CDC replays from the recorded snapshot handoff boundary"
        )
    );
    assert!(report.recommended_actions.contains(
        &"start trellara relay/apply or trellara run so CDC replays from the recorded snapshot handoff boundary"
            .to_string()
    ));
}

#[test]
fn correctness_report_surfaces_recoverable_snapshot_copy_failure() {
    let mut run = latest_snapshot_run();
    run.state = SnapshotRunState::FailedRecoverable;
    run.current_relation = Some("public.sales".to_string());
    run.failure_reason = Some("target connection dropped during table copy".to_string());

    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: None,
        latest_snapshot_run: Some(run),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.at_risk_proof_check_count, 1);
    assert_eq!(report.missing_evidence_proof_check_count, 0);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(snapshot_check.status, CorrectnessProofStatus::AtRisk);
    assert!(snapshot_check.evidence.contains("failed recoverably"));
    assert!(snapshot_check
        .evidence
        .contains("target connection dropped during table copy"));
    assert_eq!(
        snapshot_check.recommendation.as_deref(),
        Some("fix the snapshot copy failure, then resume trellara snapshot --config <flow> --run-id <run> until stream_handoff_ready")
    );
    assert!(report.recommended_actions.contains(
        &"fix the snapshot copy failure, then resume trellara snapshot --config <flow> --run-id <run> until stream_handoff_ready"
            .to_string()
    ));
}

#[test]
fn correctness_report_blocks_snapshot_handoff_watermark_mismatch() {
    let mut handoff = latest_snapshot_handoff();
    handoff.watermark_lsn = "0/16B9000".to_string();

    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(handoff),
        latest_snapshot_run: Some(latest_snapshot_run()),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.at_risk_proof_check_count, 1);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(snapshot_check.status, CorrectnessProofStatus::AtRisk);
    assert!(snapshot_check.evidence.contains("consistent LSN"));
    assert!(snapshot_check.evidence.contains("0/16B9000"));
    assert_eq!(
        snapshot_check.recommendation.as_deref(),
        Some("rerun trellara snapshot --config <flow> --force to create a fresh audited handoff")
    );
    assert!(report.recommended_actions.contains(
        &"rerun trellara snapshot --config <flow> --force to create a fresh audited handoff"
            .to_string()
    ));
}

#[test]
fn correctness_report_blocks_snapshot_handoff_identity_mismatch() {
    let mut handoff = latest_snapshot_handoff();
    handoff.dataset_id = "warehouse".to_string();
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(handoff),
        latest_snapshot_run: Some(latest_snapshot_run()),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.at_risk_proof_check_count, 1);
    let snapshot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "snapshot_handoff")
        .expect("snapshot proof check");
    assert_eq!(snapshot_check.status, CorrectnessProofStatus::AtRisk);
    assert!(snapshot_check.evidence.contains("identity does not match"));
    assert!(snapshot_check.evidence.contains("dataset_id=sales"));
    assert!(snapshot_check.evidence.contains("dataset_id=warehouse"));
    assert_eq!(
        snapshot_check.recommendation.as_deref(),
        Some("rerun trellara snapshot --config <flow> --force to create a fresh audited handoff")
    );
}
