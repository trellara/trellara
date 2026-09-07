use super::*;

#[test]
fn lake_epoch_summary_surfaces_complete_with_gaps_decision() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakeEpochSummary::from_config(
        &config,
        &LakeEpochArgs {
            config: PathBuf::from("test.yml"),
            scenario: LakeEpochScenario::OfflineStoresPublishWithGaps,
            required_source_count: 12,
            offline_source_count: 3,
            duplicate_replay_count: 2,
            format: QuickstartOutputFormat::Json,
        },
    );

    assert_eq!(summary.source_id, "local-source");
    assert_eq!(summary.dataset_id, "retail-sales");
    assert_eq!(summary.fanin_mode, "strict_envelope_epoch_fanin");
    assert_eq!(
        summary.state,
        trellara_lake::LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(summary.required_source_count, 12);
    assert_eq!(summary.complete_source_count, 9);
    assert_eq!(summary.missing_source_count, 3);
    assert_eq!(summary.straggler_policy, "publish_with_gaps");
    assert!(summary.straggler_policy_decision.contains("explicit gaps"));
    assert!(summary
        .straggler_policy_decision
        .contains("accept_complete_with_gaps"));
    assert_eq!(summary.manifest_digest.len(), 64);
    assert_eq!(summary.source_watermarks.len(), 12);
    assert_eq!(
        summary.watermark_rollup.global_low_watermark_lsn.as_deref(),
        Some("0/16B6C60")
    );
    assert_eq!(
        summary.watermark_rollup.max_source_watermark_lsn.as_deref(),
        Some("0/16B6CE0")
    );
    assert_eq!(summary.watermark_rollup.complete_source_count, 9);
    assert_eq!(summary.watermark_rollup.missing_source_count, 3);
    assert!(summary.source_watermarks.iter().any(|source| {
        source.source_id == "store-0012"
            && source.state == "missing"
            && source.gap_reason.as_deref().is_some_and(|reason| {
                reason.contains("required source missing from published gap epoch")
            })
    }));
    assert_eq!(summary.table_rollups.len(), 1);
    assert_eq!(summary.table_rollups[0].relation, "public.sales");
    assert_eq!(summary.table_rollups[0].transaction_count, 9);
    assert_eq!(summary.table_rollups[0].change_count, 9);
    assert_eq!(summary.partition_rollups.len(), 9);
    assert_eq!(summary.partition_skew.participating_partition_count, 8);
    assert_eq!(summary.partition_skew.total_event_count, 9);
    assert_eq!(summary.partition_skew.min_event_count, 1);
    assert_eq!(summary.partition_skew.max_event_count, 2);
    assert_eq!(summary.partition_skew.skew_ratio_basis_points, Some(20_000));
    assert_eq!(summary.partition_skew.hottest_partition_ids, vec![0]);
    assert_eq!(
        summary.partition_skew.coolest_partition_ids,
        vec![1, 2, 3, 4, 5, 6, 7]
    );
    assert!(summary.partition_rollups.iter().any(|partition| {
        partition.source_id == "store-0001"
            && partition.partition_id == 0
            && partition.first_commit_lsn.as_deref() == Some("0/16B6C60")
            && partition.last_commit_lsn.as_deref() == Some("0/16B6C60")
            && partition.transaction_count == 1
            && partition.event_count == 1
    }));
    assert!(summary.quarantine_entries.is_empty());
    assert_eq!(
        summary.customer_decision,
        "requires_explicit_gap_acceptance_before_spark_consumption"
    );
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("accept_complete_with_gaps")));
    assert!(summary
        .proof_command
        .contains("fleet_fanin_offline_stores_publish_with_explicit_gap_state"));
}

#[test]
fn lake_epoch_summary_surfaces_late_recovery_to_complete() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakeEpochSummary::from_config(
        &config,
        &LakeEpochArgs {
            config: PathBuf::from("test.yml"),
            scenario: LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
            required_source_count: 12,
            offline_source_count: 3,
            duplicate_replay_count: 2,
            format: QuickstartOutputFormat::Json,
        },
    );

    assert_eq!(
        summary.state,
        trellara_lake::LakeCompletenessState::Complete
    );
    assert_eq!(
        summary.recovered_state,
        Some(trellara_lake::LakeCompletenessState::Complete)
    );
    assert_eq!(summary.complete_source_count, 12);
    assert_eq!(summary.missing_source_count, 0);
    assert_eq!(summary.straggler_policy, "wait_all_required");
    assert!(summary
        .straggler_policy_decision
        .contains("recomputed complete"));
    assert_eq!(summary.manifest_digest.len(), 64);
    assert_eq!(summary.watermark_rollup.complete_source_count, 12);
    assert_eq!(summary.watermark_rollup.missing_source_count, 0);
    assert_eq!(summary.table_rollups[0].transaction_count, 12);
    assert_eq!(summary.table_rollups[0].change_count, 12);
    assert_eq!(summary.partition_rollups.len(), 12);
    assert_eq!(
        summary.customer_decision,
        "safe_for_default_spark_consumption"
    );
}

#[test]
fn lake_epoch_summary_blocks_conflicting_duplicate_quarantine() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakeEpochSummary::from_config(
        &config,
        &LakeEpochArgs {
            config: PathBuf::from("test.yml"),
            scenario: LakeEpochScenario::ConflictingDuplicateQuarantine,
            required_source_count: 12,
            offline_source_count: 3,
            duplicate_replay_count: 2,
            format: QuickstartOutputFormat::Json,
        },
    );

    assert_eq!(
        summary.state,
        trellara_lake::LakeCompletenessState::Quarantined
    );
    assert_eq!(summary.quarantined_source_count, 1);
    assert_eq!(summary.straggler_policy, "publish_with_gaps");
    assert!(summary.straggler_policy_decision.contains("quarantines"));
    assert_eq!(summary.manifest_digest.len(), 64);
    assert_eq!(summary.watermark_rollup.quarantined_source_count, 1);
    assert_eq!(summary.table_rollups[0].relation, "public.sales");
    assert_eq!(summary.quarantine_entries.len(), 1);
    assert_eq!(summary.quarantine_entries[0].source_id, "store-0001");
    assert_eq!(
        summary.quarantine_entries[0].reason,
        "conflicting_duplicate_idempotency"
    );
    assert!(summary.quarantine_entries[0]
        .recovery_command
        .contains("trellara lake fanin verify"));
    assert!(summary.source_watermarks.iter().any(|source| {
        source.state == "quarantined"
            && source
                .gap_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("conflicting duplicate"))
    }));
    assert_eq!(
        summary.customer_decision,
        "blocked_until_quarantine_is_resolved"
    );
    assert!(summary
        .injected_failure
        .as_deref()
        .is_some_and(|failure| failure.contains("conflicting transaction evidence")));
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("_trellara_quarantine")));
}
