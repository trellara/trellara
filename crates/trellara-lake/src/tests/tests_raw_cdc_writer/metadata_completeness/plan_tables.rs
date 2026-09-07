use super::*;

#[test]
fn metadata_plan_uses_reserved_raw_cdc_tables() {
    let metadata = build_epoch_metadata(input()).expect("metadata plan");

    assert_eq!(
        metadata.epochs_table,
        "retail__trellara__fanin___trellara_epochs"
    );
    assert_eq!(
        metadata.epoch_sources_table,
        "retail__trellara__fanin___trellara_epoch_sources"
    );
    assert_eq!(
        metadata.epoch_tables_table,
        "retail__trellara__fanin___trellara_epoch_tables"
    );
    assert_eq!(
        metadata.epoch_partitions_table,
        "retail__trellara__fanin___trellara_epoch_partitions"
    );
    assert_eq!(
        metadata.quarantine_table,
        "retail__trellara__fanin___trellara_quarantine"
    );
    assert_eq!(
        metadata.verification_table,
        "retail__trellara__fanin___trellara_verification"
    );
}

#[test]
fn metadata_plan_marks_epoch_complete_and_verification_matched() {
    let metadata = build_epoch_metadata(input()).expect("metadata plan");

    assert_eq!(metadata.epoch_row.state, LakeCompletenessState::Complete);
    assert_eq!(metadata.epoch_row.policy, "wait_all_required");
    assert_eq!(metadata.epoch_row.opened_at, "planned_epoch_open");
    assert_eq!(
        metadata.epoch_row.sealed_at,
        "planned_after_raw_cdc_files_durable"
    );
    assert_eq!(metadata.epoch_row.required_source_count, 0);
    assert_eq!(metadata.epoch_row.complete_source_count, 0);
    assert_eq!(metadata.epoch_row.missing_source_count, 0);
    assert_eq!(metadata.epoch_row.quarantined_source_count, 0);
    assert_eq!(metadata.epoch_row.transaction_count, 1);
    assert_eq!(metadata.epoch_row.change_count, 2);
    assert_eq!(metadata.epoch_row.checksum_rollup, 3);
    assert_eq!(metadata.epoch_row.manifest_digest.len(), 64);
    assert_eq!(metadata.epoch_row.iceberg_snapshot_id, None);
    assert!(metadata.partition_rows.is_empty());
    assert!(metadata.quarantine_rows.is_empty());
    assert_eq!(
        metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Match
    );
    assert_eq!(
        metadata.verification_row.verification_id,
        "verify:retail:epoch-1:0000000000000003"
    );
    assert_eq!(metadata.verification_row.input_transaction_count, 1);
    assert_eq!(metadata.verification_row.lake_transaction_count, 1);
}
