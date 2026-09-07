use super::*;

#[test]
fn flow_health_is_blocked_when_source_slot_is_missing() {
    let mut slot = healthy_slot();
    slot.exists = false;
    slot.issues = vec!["replication slot does not exist".to_string()];
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

    assert_eq!(health.status, FlowHealthStatus::Blocked);
    assert_eq!(
        health.issues,
        vec!["source slot slot-a: replication slot does not exist".to_string()]
    );
}

#[test]
fn flow_health_is_blocked_when_target_has_quarantine() {
    let slot = healthy_slot();
    let source = caught_up_lag();
    let target = caught_up_lag();
    let quarantine = latest_quarantine();
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &[],
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: Some(&quarantine),
        latest_validation: None,
    });

    assert_eq!(health.status, FlowHealthStatus::Blocked);
    assert_eq!(health.issue_count, 1);
    assert_eq!(
        health.issues[0],
        "target quarantine contains transaction tx-blocked at LSN 0/16B6D28: target_postgres_error"
    );
}

#[test]
fn flow_health_is_blocked_when_subscription_conflicts_exist() {
    let slot = healthy_slot();
    let conflicts = vec![subscription_stats_with_conflicts()];
    let source = caught_up_lag();
    let target = caught_up_lag();
    let health = FlowHealthSummary::from_parts(FlowHealthParts {
        source_slot: &slot,
        subscription_conflicts: &conflicts,
        source_wal_retention_warn_bytes: None,
        source: Some(&source),
        target: Some(&target),
        partition_watermarks: None,
        latest_quarantine: None,
        latest_validation: None,
    });

    assert_eq!(health.status, FlowHealthStatus::Blocked);
    assert_eq!(health.issue_count, 1);
    assert!(health.issues[0].contains("logical replication conflict"));
    assert!(health.issues[0].contains("update_missing=3"));
}
