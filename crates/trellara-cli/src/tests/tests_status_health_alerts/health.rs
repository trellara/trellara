use super::*;

#[test]
fn flow_health_is_healthy_when_slot_and_checkpoints_are_clean() {
    let slot = healthy_slot();
    let source = caught_up_lag();
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

    assert_eq!(health.status, FlowHealthStatus::Healthy);
    assert!(health.issues.is_empty());
}

#[test]
fn flow_health_degrades_when_failover_slot_is_not_synced() {
    let slot = failover_unsynced_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let validation = latest_validation(true);
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: Some(&validation),
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains("not synced to the standby"));
}

#[test]
fn flow_health_is_degraded_when_target_lags() {
    let slot = healthy_slot();
    let source = caught_up_lag();
    let target = target_lag();
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
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains("target applied LSN"));
}

#[test]
fn flow_health_is_degraded_when_partition_watermarks_are_incomplete() {
    let slot = healthy_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let watermarks = incomplete_partition_watermarks();
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: Some(&watermarks),
        latest_quarantine: None,
        latest_validation: None,
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert_eq!(health.issue_count, 1);
    assert_eq!(
        health.issues[0],
        "partition watermarks missing 2 of 4 partitions"
    );
}

#[test]
fn flow_health_is_degraded_when_latest_validation_detects_drift() {
    let slot = healthy_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let validation = latest_validation(false);
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: Some(&validation),
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert_eq!(health.issue_count, 1);
    assert_eq!(
            health.issues[0],
            "latest validation found drift in 2 of 3 tables at source LSN 0/16B9000; relations: public.sales, public.refunds"
        );
}

#[test]
fn flow_health_is_degraded_when_latest_validation_is_stale() {
    let slot = healthy_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();

    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: Some(&validation),
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains("validation watermarks are stale"));
}

#[test]
fn flow_health_is_degraded_when_source_wal_retention_exceeds_policy() {
    let slot = high_retention_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: Some(5_000),
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: None,
    });

    assert_eq!(health.status, FlowHealthStatus::Degraded);
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains(
        "source slot slot-a retained WAL 10000 bytes is at or above warning threshold 5000 bytes"
    ));
    assert!(health.issues[0].contains("restart_lsn=0/16B6B00"));
}

#[test]
fn flow_health_is_degraded_when_xmin_horizon_is_pinned() {
    let slot = xmin_horizon_slot();
    let source = caught_up_lag();
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
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains("vacuum horizon pinners"));
    assert!(health.issues[0].contains("long_running_transactions=2"));
    assert!(health.issues[0].contains("prepared_transactions=1"));
}
