use super::*;

#[test]
fn dashboard_summary_reports_clean_ready_flow() {
    let dashboard = DashboardSummary::from_status(clean_status());

    assert!(dashboard.ready);
    assert_eq!(
        dashboard.quickstart_estimated_minutes,
        QUICKSTART_ESTIMATED_MINUTES
    );
    assert_eq!(
        dashboard.quickstart_time_budget_minutes,
        QUICKSTART_TIME_BUDGET_MINUTES
    );
    assert_eq!(dashboard.status, FlowHealthStatus::Healthy);
    assert_eq!(dashboard.issue_count, 0);
    assert_eq!(dashboard.proof_check_count, 10);
    assert_eq!(dashboard.at_risk_proof_check_count, 0);
    assert_eq!(dashboard.missing_evidence_proof_check_count, 0);
    assert!(dashboard
        .proof_checks
        .iter()
        .any(|check| check.code == "checksum_validation"));
    assert_eq!(
        dashboard.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    assert!(!dashboard.transaction_boundary.manifest_barrier_required);
    assert_eq!(
        dashboard.transaction_boundary.guarantee,
        "one committed source transaction is published and applied as one ordered atomic envelope"
    );
    assert_eq!(dashboard.latest_failure, None);
    assert_eq!(
        dashboard.source_watermark_lsn,
        Some("0/16B6C50".to_string())
    );
    assert_eq!(
        dashboard.target_watermark_lsn,
        Some("0/16B6C50".to_string())
    );
    assert_eq!(dashboard.latest_validation_converged, Some(true));
    assert_eq!(dashboard.checksum_status, ChecksumStatus::Match);
    assert_eq!(
        dashboard.snapshot_handoff_status,
        CorrectnessProofStatus::Verified
    );
    assert!(dashboard
        .snapshot_handoff_evidence
        .as_ref()
        .expect("snapshot evidence")
        .contains("streaming"));
    assert_eq!(
        dashboard.latest_snapshot_handoff_relation,
        Some("public.sales".to_string())
    );
    assert_eq!(
        dashboard.latest_snapshot_state,
        Some("streaming".to_string())
    );
    assert_eq!(
        dashboard.latest_snapshot_consistent_lsn,
        Some("0/16B8000".to_string())
    );
    assert_eq!(dashboard.alert_count, 0);
    assert!(dashboard.recovery_actions.is_empty());
}

#[test]
fn dashboard_summary_surfaces_snapshot_handoff_risk() {
    let mut handoff = latest_snapshot_handoff();
    handoff.watermark_lsn = "0/16B9000".to_string();
    let dashboard = DashboardSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(handoff),
        latest_snapshot_run: Some(latest_snapshot_run()),
        ..clean_status_parts()
    }));

    assert!(!dashboard.ready);
    assert_eq!(
        dashboard.snapshot_handoff_status,
        CorrectnessProofStatus::AtRisk
    );
    assert!(dashboard
        .snapshot_handoff_evidence
        .as_ref()
        .expect("snapshot evidence")
        .contains("watermark"));
    assert_eq!(dashboard.at_risk_proof_check_count, 1);
}

#[test]
fn dashboard_summary_carries_blocked_failure_recovery_and_alerts() {
    let status = FlowStatusSummary::new(FlowStatusParts {
            latest_quarantine: Some(latest_quarantine()),
            recovery_actions: vec![FlowRecoveryAction {
                code: "target_quarantine_replay".to_string(),
                reason: "target_postgres_error".to_string(),
                transaction_id: Some("tx-blocked".to_string()),
                commit_lsn: Some("0/16B6D28".to_string()),
                command_templates: vec!["trellara quarantine replay-ready --config <config> --transaction-id tx-blocked --commit-lsn 0/16B6D28".to_string()],
                redelivery_topics: vec!["trellara.source-a.sales.strict".to_string()],
                redelivery_warnings: Vec::new(),
                hint: "seek or redeliver the strict transaction message".to_string(),
            }],
            ..clean_status_parts()
        });
    let dashboard = DashboardSummary::from_status(status);

    assert!(!dashboard.ready);
    assert_eq!(dashboard.status, FlowHealthStatus::Blocked);
    assert_eq!(
        dashboard
            .latest_failure
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("target_quarantine_blocked")
    );
    assert_eq!(dashboard.recovery_actions.len(), 1);
    assert_eq!(
        dashboard.recovery_actions[0].code,
        "target_quarantine_replay"
    );
    assert_eq!(dashboard.alert_count, 1);
    assert_eq!(
        dashboard.highest_severity,
        Some(FlowAlertSeverity::Critical)
    );
    assert_eq!(dashboard.alerts[0].code, "target_quarantine_blocked");
    assert_eq!(dashboard.checksum_status, ChecksumStatus::Match);
}
