use super::*;

mod scoring;

#[test]
fn source_safety_is_perfect_when_status_is_clean() {
    let safety = SourceSafetySummary::from_status(clean_status());

    assert_eq!(safety.score, 100);
    assert_eq!(safety.grade, SourceSafetyGrade::A);
    assert_eq!(safety.status, FlowHealthStatus::Healthy);
    assert_eq!(safety.factor_count, 0);
    assert!(safety.factors.is_empty());
    assert!(safety.recommended_actions.is_empty());
}

#[test]
fn source_safety_text_renders_executive_summary() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: ReplicationSlotStatus {
            failover: Some(true),
            synced: Some(true),
            idle_replication_slot_timeout: Some("30min".to_string()),
            ..high_retention_slot()
        },
        source_wal_retention_warn_bytes: Some(5_000),
        target: Some(target_lag()),
        ..clean_status_parts()
    }));

    let output = render_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("source safety text");

    assert!(output.contains("Trellara source safety"));
    assert!(output.contains("status: degraded"));
    assert!(output.contains("grade: C (70/100)"));
    assert!(output.contains("slot: slot-a plugin=test_decoding"));
    assert!(output.contains("failover=true"));
    assert!(output.contains("synced=true"));
    assert!(output.contains("idle_replication_slot_timeout=30min"));
    assert!(output.contains("[warning] source_wal_retention_risk"));
    assert!(output.contains("[warning] target_checkpoint_lag"));
    assert!(output.contains("recommended_actions:"));
}

#[test]
fn source_safety_html_renders_status_report_with_query_appendix() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: low_wal_headroom_slot(),
        source_wal_retention_warn_bytes: Some(1_000_000_000),
        ..clean_status_parts()
    }));

    let output = render_source_safety_summary(&safety, SourceSafetyOutputFormat::Html)
        .expect("source safety html");

    assert!(output.contains("Trellara source safety report"));
    assert!(output.contains("source-a"));
    assert!(output.contains("80/100"));
    assert!(output.contains("source_wal_retention_risk"));
    assert!(output.contains("at the current write rate"));
    assert!(output.contains("Exact Read-Only Queries"));
    assert!(output.contains("transaction_id_wraparound_budget"));
    assert!(output.contains("xmin_horizon_pinners"));
}
