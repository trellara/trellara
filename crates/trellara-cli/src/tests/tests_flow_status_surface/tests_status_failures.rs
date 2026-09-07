use super::*;

mod fixtures;
use fixtures::*;

#[test]
fn flow_status_summary_includes_source_slot_status() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
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
        latest_reseed: Some(latest_reseed()),
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(latest_snapshot_run()),
        latest_validation: Some(latest_validation(true)),
        recovery_actions: Vec::new(),
    });

    let json = serde_json::to_string(&summary).expect("json");

    assert!(json.contains("\"source_slot\""));
    assert!(json.contains("\"health\""));
    assert!(json.contains("\"retained_wal_bytes\":0"));
    assert!(json.contains("\"expected_plugin\":\"test_decoding\""));
    assert!(json.contains("\"latest_reseed\""));
    assert!(json.contains("\"watermark_lsn\":\"0/16B8000\""));
    assert!(json.contains("\"latest_snapshot_handoff\""));
    assert!(json.contains("\"relation\":\"public.sales\""));
    assert!(json.contains("\"latest_snapshot_run\""));
    assert!(json.contains("\"state\":\"streaming\""));
    assert!(json.contains("\"latest_validation\""));
    assert!(json.contains("\"source_watermark_lsn\":\"0/16B9000\""));
    assert!(json.contains("\"latest_failure\":null"));
    assert!(json.contains("\"recovery_actions\":[]"));
}

#[test]
fn flow_status_summary_includes_quarantine_recovery_action() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(latest_quarantine()),
        recovery_actions: vec![quarantine_replay_action()],
        ..clean_status_parts()
    });

    let json = serde_json::to_string(&summary).expect("json");

    assert!(json.contains("\"recovery_actions\""));
    assert!(json.contains("\"code\":\"target_quarantine_replay\""));
    assert!(json.contains("\"command_templates\""));
    assert!(json.contains("\"redelivery_topics\""));
}

#[test]
fn flow_status_summary_reports_latest_quarantine_failure() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(latest_quarantine()),
        recovery_actions: vec![quarantine_replay_action()],
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "target_quarantine_blocked");
    assert_eq!(failure.severity, FlowAlertSeverity::Critical);
    assert_eq!(
        failure.occurred_at.as_deref(),
        Some("2026-08-11 10:00:00+00")
    );
    assert_eq!(
        failure.recovery_action_codes,
        vec!["target_quarantine_replay".to_string()]
    );
    assert!(failure.message.contains("tx-blocked"));
}

#[test]
fn flow_status_summary_reports_source_schema_drift_failure() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        source_schema_drift: Some(source_schema_drift()),
        recovery_actions: vec![source_schema_handoff_action()],
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(summary.health.status, FlowHealthStatus::Blocked);
    assert!(summary
        .health
        .issues
        .iter()
        .any(|issue| issue.contains("source schema drift requires fresh handoff")));
    assert_eq!(failure.code, "source_schema_handoff_required");
    assert_eq!(failure.severity, FlowAlertSeverity::Critical);
    assert!(failure.message.contains("public.sales"));
    assert_eq!(
        failure.recovery_action_codes,
        vec!["source_schema_handoff_required".to_string()]
    );
}

#[test]
fn flow_status_summary_reports_stale_validation_failure() {
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();
    let summary = FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(validation),
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "validation_stale");
    assert_eq!(failure.severity, FlowAlertSeverity::Warning);
    assert_eq!(
        failure.occurred_at.as_deref(),
        Some("2026-08-11 10:10:00+00")
    );
    assert!(failure.message.contains("validation watermarks are stale"));
}

#[test]
fn flow_status_summary_reports_source_slot_failure_before_secondary_issues() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        source_slot: lost_wal_slot(),
        latest_quarantine: Some(latest_quarantine()),
        recovery_actions: vec![source_slot_reseed_action()],
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "source_slot_invalidated");
    assert_eq!(failure.severity, FlowAlertSeverity::Critical);
    assert!(failure.message.contains("wal_removed"));
    assert_eq!(
        failure.recovery_action_codes,
        vec!["source_slot_recreate_and_reseed".to_string()]
    );
}

#[test]
fn flow_status_summary_reports_unreserved_slot_as_warning_failure() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        source_slot: unreserved_slot(),
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "source_slot_wal_unreserved");
    assert_eq!(failure.severity, FlowAlertSeverity::Warning);
    assert!(failure.message.contains("wal_status=unreserved"));
}

#[test]
fn flow_status_summary_reports_transaction_id_wraparound_failure() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        source_slot: transaction_id_wraparound_slot(),
        target: Some(target_lag()),
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "source_transaction_id_wraparound_risk");
    assert_eq!(failure.severity, FlowAlertSeverity::Critical);
    assert!(failure.message.contains("90% used"));
    assert!(failure.message.contains("remaining_transactions=20000000"));
}
