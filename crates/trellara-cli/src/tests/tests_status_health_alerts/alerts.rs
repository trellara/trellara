use super::*;

#[test]
fn alerts_are_empty_when_status_is_clean() {
    let alerts = FlowAlertsSummary::from_status(clean_status());

    assert_eq!(alerts.status, FlowHealthStatus::Healthy);
    assert_eq!(alerts.alert_count, 0);
    assert_eq!(alerts.highest_severity, None);
    assert!(alerts.alerts.is_empty());
}

#[test]
fn alerts_report_wal_retention_warning() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: high_retention_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(5_000),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_wal_retention_risk");
    assert_eq!(alerts.alerts[0].severity, FlowAlertSeverity::Warning);
    assert!(alerts.alerts[0]
        .message
        .contains("retained WAL 10000 bytes"));
    assert!(alerts.alerts[0].message.contains("restart_lsn=0/16B6B00"));
    assert!(alerts.alerts[0]
        .message
        .contains("confirmed_flush_lsn=0/16B6B00"));
}

#[test]
fn alerts_report_projected_wal_headroom_warning() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: low_wal_headroom_slot(),
        source_wal_retention_warn_bytes: Some(1_000_000_000),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_wal_retention_risk");
    assert!(alerts.alerts[0].message.contains("~14 hours"));
    assert!(alerts.alerts[0].message.contains("current write rate"));
}

#[test]
fn alerts_report_transaction_id_wraparound_risk() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: transaction_id_wraparound_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Blocked);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Critical));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(
        alerts.alerts[0].code,
        "source_transaction_id_wraparound_risk"
    );
    assert!(alerts.alerts[0].message.contains("90% used"));
    assert!(alerts.alerts[0]
        .recommendation
        .contains("source writes offline"));
}

#[test]
fn alerts_recommend_reseed_when_source_slot_lost_wal() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: lost_wal_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Blocked);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Critical));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_slot_invalidated");
    assert!(alerts.alerts[0].recommendation.contains("reseed"));
    assert!(alerts.alerts[0].recommendation.contains("fresh handoff"));
}

#[test]
fn alerts_report_unreserved_source_slot_as_warning() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: unreserved_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_slot_wal_unreserved");
    assert!(alerts.alerts[0].message.contains("wal_status=unreserved"));
    assert!(alerts.alerts[0].message.contains("restart_lsn=0/16B6B00"));
}

#[test]
fn alerts_report_failover_slot_sync_as_warning() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: failover_unsynced_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_slot_failover_not_synced");
    assert!(alerts.alerts[0]
        .message
        .contains("not synced to the standby"));
    assert!(alerts.alerts[0].message.contains("failover=true"));
    assert!(alerts.alerts[0].message.contains("synced=false"));
    assert!(alerts.alerts[0].message.contains("restart_lsn=0/16B6B00"));
    assert!(alerts.alerts[0]
        .recommendation
        .contains("standby slot synchronization"));
}

#[test]
fn alerts_report_subscription_conflicts_as_critical() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        subscription_conflicts: vec![subscription_stats_with_conflicts()],
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Blocked);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Critical));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "logical_replication_conflicts");
    assert!(alerts.alerts[0].message.contains("downstream_sales"));
    assert!(alerts.alerts[0].message.contains("update_missing=3"));
    assert!(alerts.alerts[0].recommendation.contains("reseed"));
}

#[test]
fn alerts_report_source_schema_drift_as_critical() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_schema_drift: Some(source_schema_drift()),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Blocked);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Critical));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "source_schema_handoff_required");
    assert!(alerts.alerts[0].message.contains("public.sales"));
    assert!(alerts.alerts[0].recommendation.contains("schema-discover"));
}

#[test]
fn alerts_report_quarantine_as_critical() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(latest_quarantine()),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Blocked);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Critical));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "target_quarantine_blocked");
    assert!(alerts.alerts[0]
        .recommendation
        .contains("quarantine replay-ready"));
}

#[test]
fn alerts_report_incomplete_partition_watermarks() {
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "partition_watermark_incomplete");
    assert!(alerts.alerts[0]
        .recommendation
        .contains("barrier-aware applier"));
}

#[test]
fn alerts_report_stale_validation_as_warning() {
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();
    let alerts = FlowAlertsSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(validation),
        ..clean_status_parts()
    }));

    assert_eq!(alerts.status, FlowHealthStatus::Degraded);
    assert_eq!(alerts.highest_severity, Some(FlowAlertSeverity::Warning));
    assert_eq!(alerts.alerts.len(), 1);
    assert_eq!(alerts.alerts[0].code, "validation_stale");
    assert!(alerts.alerts[0]
        .message
        .contains("validation watermarks are stale"));
    assert!(alerts.alerts[0].recommendation.contains("trellara verify"));
}
