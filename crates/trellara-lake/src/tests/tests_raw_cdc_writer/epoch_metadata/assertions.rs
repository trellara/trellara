use super::*;

pub(super) fn assert_commit_steps(plan: &LakeRawCdcEpochWritePlan) {
    assert_eq!(plan.commit_steps.len(), 6);
    assert_eq!(plan.commit_steps[0].order, 10);
    assert_eq!(plan.commit_steps[0].phase, "raw_cdc_data");
    assert!(plan.commit_steps[0]
        .action
        .contains("append 2 changes for public.sales"));
    assert!(plan.commit_steps[0]
        .durability_gate
        .contains("before source acknowledgement"));
    assert!(plan.commit_steps[0]
        .recovery_rule
        .contains("replay this data-file intent idempotently"));
    assert_eq!(plan.commit_steps[1].phase, "epoch_sources_metadata");
    assert_eq!(plan.commit_steps[2].phase, "epoch_tables_metadata");
    assert_eq!(plan.commit_steps[3].phase, "epoch_partitions_metadata");
    assert_eq!(plan.commit_steps[4].phase, "epoch_row_metadata");
    assert_eq!(plan.commit_steps[5].phase, "verification_metadata");
    assert!(plan.commit_steps[5]
        .durability_gate
        .contains("Iceberg checkpoint receipts"));
    assert!(plan.commit_steps[5]
        .recovery_rule
        .contains("verify checkpoint receipts"));
    assert!(plan.commit_steps[5]
        .recovery_rule
        .contains("discover epoch epoch-2026-08-16T06 metadata"));
}

pub(super) fn assert_recovery_scenarios(plan: &LakeRawCdcEpochWritePlan) {
    assert_eq!(plan.recovery_scenarios.len(), 5);
    assert!(plan.recovery_scenarios.iter().any(|scenario| {
        scenario.code == "writer_crash_before_data_file_commit"
            && scenario.replay_policy == "retry_data_file_intents"
            && scenario
                .recovery_action
                .contains("Trellara idempotency keys")
    }));
    assert!(plan.recovery_scenarios.iter().any(|scenario| {
        scenario.code == "writer_crash_after_data_before_epoch_metadata"
            && scenario
                .replay_policy
                .contains("checkpoint_receipts_then_publish_metadata")
            && scenario
                .recovery_action
                .contains("Iceberg checkpoint receipts")
    }));
    assert!(plan.recovery_scenarios.iter().any(|scenario| {
        scenario.code == "writer_crash_after_epoch_metadata"
            && scenario.replay_policy == "discover_existing_epoch_metadata_before_publish"
            && scenario
                .recovery_action
                .contains("verify checkpoint receipts")
    }));
    assert!(plan.recovery_scenarios.iter().any(|scenario| {
        scenario.code == "verification_mismatch_after_commit"
            && scenario.replay_policy == "hold_spark_consumption_until_fanin_verify_matches"
            && scenario.recovery_action.contains("hold Spark consumption")
    }));
}

pub(super) fn assert_data_file(
    plan: &LakeRawCdcEpochWritePlan,
    first: &TransactionEnvelope,
    second: &TransactionEnvelope,
) {
    let file = &plan.data_files[0];
    assert_eq!(file.table_name, "retail__public__sales__raw_cdc");
    assert_eq!(file.relation, "public.sales");
    assert_eq!(file.source_bucket, 0);
    assert_eq!(file.source_ids, vec!["store-001", "store-002"]);
    assert_eq!(file.transaction_count, 2);
    assert_eq!(file.change_count, 2);
    assert_eq!(file.min_commit_lsn, "0/16B6C50");
    assert_eq!(file.max_commit_lsn, "0/16B6C70");
    assert_eq!(file.idempotency_key_count, 2);
    assert_eq!(file.checksum_rollup, first.checksum ^ second.checksum);
    assert_eq!(
        file.object_key_hint,
        "retail__public__sales__raw_cdc/epoch_id=epoch-2026-08-16T06/source_bucket=0000/part-00000.parquet"
    );
}

pub(super) fn assert_first_row_intent(
    plan: &LakeRawCdcEpochWritePlan,
    first: &TransactionEnvelope,
) {
    assert_eq!(plan.row_intents.len(), 2);
    let first_row = &plan.row_intents[0];
    assert_eq!(first_row.source_id, "store-001");
    assert_eq!(first_row.database_id, "postgres");
    assert_eq!(first_row.dataset_id, "retail");
    assert_eq!(first_row.relation, "public.sales");
    assert_eq!(first_row.transaction_id, "tx-1");
    assert_eq!(first_row.begin_lsn, "0/16B6B00");
    assert_eq!(first_row.commit_lsn, "0/16B6C50");
    assert_eq!(first_row.total_order, 1);
    assert_eq!(first_row.operation, "insert");
    assert_eq!(first_row.record_key.as_deref(), Some("sale-1"));
    assert_eq!(first_row.idempotency_key, first.changes[0].idempotency_key);
    assert_eq!(first_row.schema_fingerprint, None);
    assert_eq!(first_row.envelope_checksum, first.checksum);
    assert_eq!(first_row.manifest_id, None);
    assert_eq!(first_row.manifest_boundary_mode, None);
    assert_eq!(first_row.manifest_global_event_count, None);
    assert_eq!(first_row.manifest_participating_partition_count, None);
    assert_eq!(first_row.partition_key, None);
    assert_eq!(first_row.epoch_id, "epoch-2026-08-16T06");
    assert_eq!(first_row.ingested_at, "planned_after_raw_cdc_files_durable");
    assert!(first_row.payload_before.is_empty());
    assert_eq!(first_row.payload_after.len(), 2);
    assert_eq!(first_row.payload_after[0].name, "id");
}

pub(super) fn assert_epoch_metadata(plan: &LakeRawCdcEpochWritePlan, checksum_rollup: u64) {
    assert_eq!(
        plan.epoch_metadata.epochs_table,
        "retail__trellara__fanin___trellara_epochs"
    );
    assert_eq!(
        plan.epoch_metadata.epoch_sources_table,
        "retail__trellara__fanin___trellara_epoch_sources"
    );
    assert_eq!(
        plan.epoch_metadata.epoch_tables_table,
        "retail__trellara__fanin___trellara_epoch_tables"
    );
    assert_eq!(
        plan.epoch_metadata.epoch_partitions_table,
        "retail__trellara__fanin___trellara_epoch_partitions"
    );
    assert_eq!(
        plan.epoch_metadata.quarantine_table,
        "retail__trellara__fanin___trellara_quarantine"
    );
    assert_eq!(
        plan.epoch_metadata.verification_table,
        "retail__trellara__fanin___trellara_verification"
    );
    assert_eq!(
        plan.epoch_metadata.epoch_row.state,
        LakeCompletenessState::Complete
    );
    assert_eq!(plan.epoch_metadata.epoch_row.policy, "wait_all_required");
    assert_eq!(plan.epoch_metadata.epoch_row.required_source_count, 2);
    assert_eq!(plan.epoch_metadata.epoch_row.complete_source_count, 2);
    assert_eq!(plan.epoch_metadata.epoch_row.missing_source_count, 0);
    assert_eq!(plan.epoch_metadata.epoch_row.quarantined_source_count, 0);
    assert_eq!(plan.epoch_metadata.epoch_row.manifest_digest.len(), 64);
    assert_eq!(plan.epoch_metadata.epoch_row.iceberg_snapshot_id, None);
    assert!(plan.epoch_metadata.quarantine_rows.is_empty());
    assert_eq!(plan.epoch_metadata.source_rows.len(), 2);
    assert!(plan.epoch_metadata.partition_rows.is_empty());
    assert!(plan
        .epoch_metadata
        .source_rows
        .iter()
        .all(|source| source.state == LakeEpochSourceState::Complete));
    assert_eq!(plan.epoch_metadata.table_rows[0].relation, "public.sales");
    assert_eq!(plan.epoch_metadata.table_rows[0].transaction_count, 2);
    assert_eq!(
        plan.epoch_metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Match
    );
    assert_eq!(
        plan.epoch_metadata.verification_row.verification_id,
        format!("verify:retail:epoch-2026-08-16T06:{checksum_rollup:016x}")
    );
    assert_eq!(
        plan.epoch_metadata.verification_row.completed_at,
        "planned_after_raw_cdc_files_durable"
    );
}
