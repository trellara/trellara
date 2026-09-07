use super::*;

#[test]
fn direct_source_safety_text_includes_read_only_table_and_slot_context() {
    let init_recommendation = SourceSafetyInitRecommendation {
            command: "trellara init --source-database-url postgresql://source/app --target-database-url <target-postgres-url> --source-id source-a --dataset-id sales --table public.sales --primary-key id --output trellara.yml --evaluate".to_string(),
            output: "trellara.yml".to_string(),
            table_count: 1,
            primary_key: "id".to_string(),
            evaluation_ready: true,
            note: "replace target".to_string(),
        };
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        active_healthy_slot(),
        Vec::new(),
        Some(init_recommendation),
    );

    let output = render_direct_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("direct source safety text");

    assert!(output.contains("read_only: true"));
    assert!(output.contains("tables: 1 checked, 0 unsafe"));
    assert!(output.contains("slot: slot-a plugin=test_decoding active=true"));
    assert!(output.contains("restart_lsn=0/16B6B00"));
    assert!(output.contains("confirmed_flush_lsn=0/16B6B00"));
    assert!(output.contains("wal_status=reserved"));
    assert!(output.contains("safe_wal_size_bytes=1000000"));
    assert!(output.contains("findings:\n- none"));
    assert!(output.contains("recommended_actions:\n- none"));
    assert!(output.contains("init_recommendation:"));
    assert!(output.contains("trellara init --source-database-url"));
    assert!(output.contains("--target-database-url <target-postgres-url>"));
    assert!(output.contains("--evaluate"));
    assert!(output.contains("tables: 1 primary_key=id evaluation_ready=true"));
}

#[test]
fn direct_source_safety_html_is_shareable_dba_report() {
    let init_recommendation = SourceSafetyInitRecommendation {
        command: "trellara init --source-database-url postgresql://source/app --target-database-url <target-postgres-url> --table public.sales --output trellara.yml --evaluate".to_string(),
        output: "trellara.yml".to_string(),
        table_count: 1,
        primary_key: "id".to_string(),
        evaluation_ready: true,
        note: "replace target".to_string(),
    };
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source<prod>".to_string(),
        "sales".to_string(),
        Some(1_000_000_000),
        vec![preflight_table(42)],
        low_wal_headroom_slot(),
        vec![subscription_stats_with_conflicts()],
        Some(init_recommendation),
    );

    let output = render_direct_source_safety_summary(&safety, SourceSafetyOutputFormat::Html)
        .expect("direct source safety html");

    assert!(output.starts_with("<!doctype html>"));
    assert!(output.contains("<title>Trellara source safety report</title>"));
    assert!(output.contains("source&lt;prod&gt;"));
    assert!(output.contains("45/100"));
    assert!(output.contains("source_wal_retention_risk"));
    assert!(output.contains("~14 hours"));
    assert!(output.contains("Recommended Actions"));
    assert!(output.contains("downstream_sales apply_errors=2"));
    assert!(output.contains("&lt;target-postgres-url&gt;"));
    assert!(output.contains("Exact Read-Only Queries"));
    assert!(output.contains("pg_current_wal_lsn()"));
    assert!(output.contains("pg_replication_slots"));
}

#[test]
fn direct_source_safety_text_surfaces_extended_slot_posture() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        ReplicationSlotStatus {
            active: Some(false),
            failover: Some(true),
            synced: Some(false),
            inactive_since: Some("2026-08-12 10:00:00+00".to_string()),
            idle_replication_slot_timeout: Some("0".to_string()),
            invalidation_reason: Some("wal_removed".to_string()),
            ..healthy_slot()
        },
        Vec::new(),
        None,
    );

    let output = render_direct_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("direct source safety text");

    assert!(output.contains("invalidation_reason=wal_removed"));
    assert!(output.contains("failover=true"));
    assert!(output.contains("synced=false"));
    assert!(output.contains("inactive_since=2026-08-12 10:00:00+00"));
    assert!(output.contains("idle_replication_slot_timeout=0"));
}

#[test]
fn direct_source_safety_blocks_on_logical_replication_conflicts() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        active_healthy_slot(),
        vec![subscription_stats_with_conflicts()],
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.score, 65);
    assert_eq!(safety.critical_factor_count, 1);
    assert_eq!(safety.warning_factor_count, 0);
    assert_eq!(safety.subscription_conflicts.len(), 1);
    assert_eq!(safety.factors[0].code, "logical_replication_conflicts");
    assert!(safety.factors[0]
        .evidence
        .contains("6 logical replication conflict"));
    assert!(safety.factors[0].evidence.contains("update_missing=3"));
    assert!(safety.recommended_actions[0].contains("reseed"));

    let output = render_direct_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("direct source safety text");

    assert!(output.contains("subscription_conflicts:"));
    assert!(output.contains("downstream_sales apply_errors=2 sync_errors=0 conflicts=6"));
    assert!(output.contains("update_missing=3"));
    assert!(output.contains("delete_missing=2"));
}

#[test]
fn direct_source_safety_warns_on_unclassified_subscription_apply_errors() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        active_healthy_slot(),
        vec![subscription_stats_with_apply_errors()],
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 85);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "logical_replication_apply_errors");
    assert!(safety.factors[0].evidence.contains("apply_error_count=4"));
    assert!(safety.recommended_actions[0].contains("verification"));

    let output = render_direct_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("direct source safety text");

    assert!(output.contains("subscription_conflicts:"));
    assert!(output.contains("downstream_inventory apply_errors=4 sync_errors=1 conflicts=0"));
}
