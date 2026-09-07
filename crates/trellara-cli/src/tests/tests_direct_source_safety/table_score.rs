use super::*;

#[test]
fn direct_source_safety_scores_table_and_slot_risk() {
    let unsafe_table = trellara_pg_capture::TablePreflight {
        update_delete_safe: false,
        issues: vec!["table has no primary key".to_string()],
        ..preflight_table(42)
    };
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000),
        vec![unsafe_table],
        ReplicationSlotStatus {
            retained_wal_bytes: Some(10_000),
            ..lost_wal_slot()
        },
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.unsafe_table_count, 1);
    assert_eq!(safety.critical_factor_count, 2);
    assert_eq!(
        safety
            .factors
            .iter()
            .map(|factor| factor.code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "table_cdc_unsafe",
            "source_slot_invalidated",
            "source_wal_retention_risk"
        ]
    );
    assert_eq!(safety.grade, SourceSafetyGrade::F);
}

#[test]
fn direct_source_safety_is_healthy_when_tables_and_active_slot_are_clean() {
    let safety = DirectSourceSafetySummary::from_source_inspection(
        "source-a".to_string(),
        "sales".to_string(),
        Some(1_000_000),
        vec![preflight_table(42)],
        active_healthy_slot(),
        Vec::new(),
        None,
    );

    assert_eq!(safety.status, FlowHealthStatus::Healthy);
    assert_eq!(safety.score, 100);
    assert_eq!(safety.factor_count, 0);
}
