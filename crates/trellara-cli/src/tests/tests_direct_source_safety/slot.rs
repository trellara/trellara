use super::*;

#[test]
fn direct_source_safety_treats_missing_slot_as_pre_bootstrap_warning() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        missing_slot(),
        Vec::new(),
        None,
    );

    assert!(safety.read_only);
    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 95);
    assert_eq!(safety.grade, SourceSafetyGrade::A);
    assert_eq!(safety.critical_factor_count, 0);
    assert_eq!(safety.warning_factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_not_created");
    assert!(safety.factors[0].evidence.contains("restart_lsn=unknown"));
    assert!(safety.factors[0]
        .evidence
        .contains("confirmed_flush_lsn=unknown"));
    assert!(safety.factors[0].evidence.contains("wal_status=unknown"));
}

#[test]
fn direct_source_safety_flags_abandoned_slot_guidance() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        healthy_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factors[0].code, "source_slot_inactive");
    assert!(safety.recommended_actions[0].contains("abandoned slots"));
}

#[test]
fn direct_source_safety_mentions_disabled_idle_slot_timeout() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        ReplicationSlotStatus {
            inactive_since: Some("2026-08-12 10:00:00+00".to_string()),
            idle_replication_slot_timeout: Some("0".to_string()),
            ..healthy_slot()
        },
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factors[0].code, "source_slot_inactive");
    assert!(safety.factors[0].evidence.contains("inactive since"));
    assert!(safety.factors[0].evidence.contains("restart_lsn=0/16B6B00"));
    assert!(safety.factors[0]
        .evidence
        .contains("inactive_since=2026-08-12 10:00:00+00"));
    assert!(safety.factors[0]
        .evidence
        .contains("idle_replication_slot_timeout=0"));
    assert!(safety.recommended_actions[0].contains("idle_replication_slot_timeout"));
}

#[test]
fn direct_source_safety_surfaces_configured_idle_slot_timeout() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        ReplicationSlotStatus {
            inactive_since: Some("2026-08-12 10:00:00+00".to_string()),
            idle_replication_slot_timeout: Some("1h".to_string()),
            ..healthy_slot()
        },
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factors[0].code, "source_slot_inactive");
    assert!(safety.factors[0]
        .evidence
        .contains("idle_replication_slot_timeout is 1h"));
    assert!(safety.recommended_actions[0].contains("cleanup is configured"));
}

#[test]
fn direct_source_safety_reports_plugin_mismatch_as_specific_factor() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        plugin_mismatch_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_plugin_mismatch");
    assert!(safety.factors[0].evidence.contains("expected pgoutput"));
    assert!(safety.recommended_actions[0].contains("expected logical decoding plugin"));
}

#[test]
fn direct_source_safety_warns_when_failover_slot_is_disabled() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        failover_disabled_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 85);
    assert_eq!(safety.grade, SourceSafetyGrade::B);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_failover_disabled");
    assert!(safety.factors[0]
        .evidence
        .contains("not configured as a failover slot"));
    assert!(safety.factors[0].evidence.contains("failover=false"));
    assert!(safety.factors[0].evidence.contains("restart_lsn=0/16B6B00"));
    assert!(safety.recommended_actions[0].contains("failover logical slot"));
}

#[test]
fn direct_source_safety_includes_pg18_slot_fields_in_wal_evidence() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000),
        vec![preflight_table(42)],
        ReplicationSlotStatus {
            failover: Some(true),
            synced: Some(false),
            idle_replication_slot_timeout: Some("30min".to_string()),
            ..high_retention_slot()
        },
        Vec::new(),
        None,
    );

    let wal_retention = safety
        .factors
        .iter()
        .find(|factor| factor.code == "source_wal_retention_risk")
        .expect("wal retention factor");

    assert!(wal_retention.evidence.contains("failover=true"));
    assert!(wal_retention.evidence.contains("synced=false"));
    assert!(wal_retention
        .evidence
        .contains("idle_replication_slot_timeout=30min"));
}

#[test]
fn direct_source_safety_reports_projected_wal_headroom() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000_000),
        vec![preflight_table(42)],
        low_wal_headroom_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_wal_retention_risk");
    assert!(safety.factors[0].evidence.contains("~14 hours"));
    assert!(safety.factors[0]
        .evidence
        .contains("at the current write rate"));
    assert!(safety.factors[0]
        .evidence
        .contains("wal_headroom=~14 hours at 1000 bytes/sec over 2000ms"));
}

#[test]
fn direct_source_safety_flags_transaction_id_wraparound_risk() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        transaction_id_wraparound_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.critical_factor_count, 1);
    assert_eq!(
        safety.factors[0].code,
        "source_transaction_id_wraparound_risk"
    );
    assert!(safety.factors[0].evidence.contains("90% used"));
    assert!(safety.factors[0]
        .evidence
        .contains("remaining_transactions=20000000"));
    assert!(safety.recommended_actions[0].contains("force source writes offline"));
}

#[test]
fn direct_source_safety_flags_xmin_horizon_pinners() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        None,
        vec![preflight_table(42)],
        xmin_horizon_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.warning_factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_xmin_horizon_risk");
    assert!(safety.factors[0]
        .evidence
        .contains("vacuum horizon pinners"));
    assert!(safety.factors[0]
        .evidence
        .contains("stale_catalog_xmin_slots=1"));
    assert!(safety.factors[0]
        .evidence
        .contains("long_running_transactions=2"));
    assert!(safety.factors[0]
        .evidence
        .contains("prepared_transactions=1"));
    assert!(safety.factors[0]
        .evidence
        .contains("hot_standby_feedback_replicas=1"));
}
