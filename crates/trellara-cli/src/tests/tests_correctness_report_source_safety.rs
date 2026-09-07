use super::*;

#[test]
fn correctness_report_is_not_ready_when_source_wal_retention_exceeds_policy() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: high_retention_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(5_000),
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
        latest_validation: Some(latest_validation(true)),
        recovery_actions: Vec::new(),
    }));

    assert!(!report.ready);
    assert!(!report.source_wal_retention_safe);
    assert_eq!(report.issues.len(), 1);
    assert!(report.issues[0].contains("retained WAL 10000 bytes"));
    assert_eq!(
            report.recommended_actions,
            vec![
                "drain relay/apply lag or reseed slow targets before source WAL retention grows further"
            ]
        );
}

#[test]
fn correctness_report_is_not_ready_when_wal_headroom_is_low() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: low_wal_headroom_slot(),
        source_wal_retention_warn_bytes: Some(1_000_000_000),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert!(!report.source_wal_retention_safe);
    assert!(report.issues[0].contains("~14 hours"));
    let check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "source_wal_retention")
        .expect("source wal retention proof check");
    assert_eq!(check.status, CorrectnessProofStatus::AtRisk);
    assert!(check.evidence.contains("current write rate"));
}

#[test]
fn correctness_report_is_not_ready_when_xmin_horizon_is_pinned() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: xmin_horizon_slot(),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert!(!report.source_slot_safe);
    assert!(report.issues[0].contains("vacuum horizon pinners"));
    let check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "source_slot")
        .expect("source slot proof check");
    assert_eq!(check.status, CorrectnessProofStatus::AtRisk);
    assert!(check.evidence.contains("prepared_transactions=1"));
    assert!(report.recommended_actions[0].contains("hot_standby_feedback"));
}

#[test]
fn correctness_report_recommends_reseed_when_source_slot_lost_wal() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: lost_wal_slot(),
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
        latest_validation: Some(latest_validation(true)),
        recovery_actions: Vec::new(),
    }));

    assert!(!report.ready);
    assert!(!report.source_slot_safe);
    assert_eq!(report.issues.len(), 2);
    assert!(report.issues[0].contains("invalidated"));
    let source_slot_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "source_slot")
        .expect("source slot proof check");
    assert_eq!(source_slot_check.status, CorrectnessProofStatus::AtRisk);
    assert_eq!(
        source_slot_check.issue_code.as_deref(),
        Some("source_slot_at_risk")
    );
    assert!(source_slot_check.evidence.contains("wal_removed"));
    assert_eq!(report.recommended_actions.len(), 1);
    assert!(report.recommended_actions[0].contains("reseed"));
    assert!(report.recommended_actions[0].contains("fresh handoff"));
}

#[test]
fn correctness_report_blocks_on_subscription_conflicts() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        subscription_conflicts: vec![subscription_stats_with_conflicts()],
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert!(!report.source_subscription_conflicts_safe);
    assert_eq!(report.at_risk_proof_check_count, 1);
    assert_eq!(report.issues.len(), 1);
    assert!(report.issues[0].contains("logical replication conflict"));
    let conflict_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "source_subscription_conflicts")
        .expect("subscription conflict proof check");
    assert_eq!(conflict_check.status, CorrectnessProofStatus::AtRisk);
    assert_eq!(
        conflict_check.issue_code.as_deref(),
        Some("source_subscription_conflicts_at_risk")
    );
    assert!(conflict_check.evidence.contains("downstream_sales"));
    assert!(conflict_check.evidence.contains("update_missing=3"));
    assert_eq!(report.recommended_actions.len(), 1);
    assert!(report.recommended_actions[0].contains("subscriber-side divergence"));
    assert!(report.recommended_actions[0].contains("reseed"));
}

#[test]
fn correctness_report_requires_fresh_handoff_on_source_schema_drift() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_schema_drift: Some(source_schema_drift()),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert!(!report.source_schema_contract_safe);
    assert_eq!(
        report
            .latest_failure
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("source_schema_handoff_required")
    );
    let schema_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "source_schema_contract")
        .expect("schema contract proof check");
    assert_eq!(schema_check.status, CorrectnessProofStatus::AtRisk);
    assert!(schema_check.evidence.contains("public.sales"));
    assert_eq!(report.recommended_actions.len(), 1);
    assert!(report.recommended_actions[0].contains("public.sales"));
    assert!(report.recommended_actions[0].contains("schema-discover"));
    assert!(report.recommended_actions[0].contains("snapshot-to-stream"));
}
