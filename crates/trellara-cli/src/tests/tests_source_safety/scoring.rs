use super::*;

#[test]
fn source_safety_scores_retention_and_target_lag() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: high_retention_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(5_000),
        target: Some(target_lag()),
        ..clean_status_parts()
    }));

    assert_eq!(safety.score, 70);
    assert_eq!(safety.grade, SourceSafetyGrade::C);
    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factor_count, 2);
    assert_eq!(safety.critical_factor_count, 0);
    assert_eq!(safety.warning_factor_count, 2);
    assert_eq!(
        safety
            .factors
            .iter()
            .map(|factor| factor.code.as_str())
            .collect::<Vec<_>>(),
        vec!["source_wal_retention_risk", "target_checkpoint_lag"]
    );
    assert_eq!(safety.recommended_actions.len(), 2);
}

#[test]
fn source_safety_scores_projected_wal_headroom() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: low_wal_headroom_slot(),
        source_wal_retention_warn_bytes: Some(1_000_000_000),
        ..clean_status_parts()
    }));

    assert_eq!(safety.score, 80);
    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_wal_retention_risk");
    assert!(safety.factors[0].evidence.contains("~14 hours"));
    assert!(safety.factors[0].evidence.contains("current write rate"));
}

#[test]
fn source_safety_warns_when_target_is_behind_source_durable_watermark() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source: Some(CheckpointLag {
            last_seen_lsn: "0/16B8000".to_string(),
            last_durable_lsn: "0/16B8000".to_string(),
            source_is_durable: true,
            ..caught_up_lag()
        }),
        target: Some(caught_up_lag()),
        ..clean_status_parts()
    }));

    assert_eq!(safety.score, 90);
    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.factors.len(), 1);
    assert_eq!(safety.factors[0].code, "target_source_watermark_lag");
    assert!(safety.factors[0]
        .evidence
        .contains("behind source durable LSN 0/16B8000"));
    assert!(safety.recommended_actions[0].contains("source durable watermark"));
}

#[test]
fn source_safety_scores_lost_source_wal_as_critical() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: lost_wal_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.score, 65);
    assert_eq!(safety.grade, SourceSafetyGrade::C);
    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.critical_factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_invalidated");
    assert!(safety.factors[0].evidence.contains("wal_removed"));
    assert!(safety.recommended_actions[0].contains("reseed"));
}

#[test]
fn source_safety_missing_slot_evidence_includes_position_contract() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: missing_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.factors[0].code, "source_slot_missing");
    assert!(safety.factors[0].evidence.contains("restart_lsn=unknown"));
    assert!(safety.factors[0]
        .evidence
        .contains("confirmed_flush_lsn=unknown"));
    assert!(safety.factors[0].evidence.contains("wal_status=unknown"));
}

#[test]
fn source_safety_distinguishes_lost_wal_without_invalidation() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: wal_lost_without_invalidation_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_wal_lost");
    assert!(safety.factors[0].evidence.contains("wal_status=lost"));
}

#[test]
fn source_safety_warns_before_unreserved_slot_loses_wal() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: unreserved_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 80);
    assert_eq!(safety.factors[0].code, "source_slot_wal_unreserved");
    assert!(safety.factors[0].evidence.contains("restart_lsn=0/16B6B00"));
    assert!(safety.factors[0]
        .evidence
        .contains("confirmed_flush_lsn=0/16B6B00"));
    assert!(safety.recommended_actions[0].contains("drain"));
}

#[test]
fn source_safety_warns_when_safe_wal_size_is_exhausted() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: exhausted_safe_wal_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 80);
    assert_eq!(safety.factors[0].code, "source_slot_safe_wal_exhausted");
    assert!(safety.factors[0].evidence.contains("safe_wal_size"));
    assert!(safety.factors[0].evidence.contains("retained_wal_bytes=0"));
}

#[test]
fn source_safety_warns_when_failover_slot_is_not_synced() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: failover_unsynced_slot(),
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Degraded);
    assert_eq!(safety.score, 85);
    assert_eq!(safety.grade, SourceSafetyGrade::B);
    assert_eq!(safety.factor_count, 1);
    assert_eq!(safety.factors[0].code, "source_slot_failover_not_synced");
    assert_eq!(safety.slot.failover, Some(true));
    assert_eq!(safety.slot.synced, Some(false));
    assert!(safety.factors[0].evidence.contains("not synced"));
    assert!(safety.factors[0].evidence.contains("failover=true"));
    assert!(safety.factors[0].evidence.contains("synced=false"));
    assert!(safety.factors[0].evidence.contains("restart_lsn=0/16B6B00"));
    assert!(safety.recommended_actions[0].contains("standby slot synchronization"));
}

#[test]
fn source_safety_warns_when_failover_slot_is_disabled() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        source_slot: failover_disabled_slot(),
        ..clean_status_parts()
    }));

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
fn source_safety_blocks_on_configured_subscription_conflicts() {
    let safety = SourceSafetySummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        subscription_conflicts: vec![subscription_stats_with_conflicts()],
        ..clean_status_parts()
    }));

    assert_eq!(safety.status, FlowHealthStatus::Blocked);
    assert_eq!(safety.score, 65);
    assert_eq!(safety.critical_factor_count, 1);
    assert_eq!(safety.factors[0].code, "logical_replication_conflicts");
    assert_eq!(safety.subscription_conflicts.len(), 1);
    assert!(safety.factors[0].evidence.contains("update_missing=3"));

    let output = render_source_safety_summary(&safety, SourceSafetyOutputFormat::Text)
        .expect("source safety text");

    assert!(output.contains("subscription_conflicts:"));
    assert!(output.contains("downstream_sales apply_errors=2 sync_errors=0 conflicts=6"));
    assert!(output.contains("[critical] logical_replication_conflicts"));
}
