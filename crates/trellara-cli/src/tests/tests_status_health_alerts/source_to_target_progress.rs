use super::*;

#[test]
fn flow_health_reports_target_behind_source_durable_watermark() {
    let slot = healthy_slot();
    let source = source_at_newer_durable_lsn();
    let target = caught_up_lag();

    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: None,
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert!(health
        .issues
        .iter()
        .any(|issue| issue.contains("behind source durable LSN 0/16B8000")));
}

#[test]
fn alerts_report_target_behind_source_durable_watermark() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source: Some(source_at_newer_durable_lsn()),
        target: Some(caught_up_lag()),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "target_source_watermark_lag");
    assert!(alerts.alerts[0]
        .message
        .contains("behind source durable LSN 0/16B8000"));
    assert!(alerts.alerts[0]
        .recommendation
        .contains("source durable watermark"));
}

fn source_at_newer_durable_lsn() -> CheckpointLag {
    CheckpointLag {
        last_seen_lsn: "0/16B8000".to_string(),
        last_durable_lsn: "0/16B8000".to_string(),
        source_is_durable: true,
        ..caught_up_lag()
    }
}
