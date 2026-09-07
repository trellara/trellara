use super::*;

#[test]
fn raw_cdc_epoch_metadata_rows_round_trip_as_json_artifacts() {
    let metadata = LakeRawCdcEpochMetadataPlan {
        epochs_table: "retail__trellara__fanin___trellara_epochs".to_string(),
        epoch_sources_table: "retail__trellara__fanin___trellara_epoch_sources".to_string(),
        epoch_tables_table: "retail__trellara__fanin___trellara_epoch_tables".to_string(),
        epoch_partitions_table: "retail__trellara__fanin___trellara_epoch_partitions".to_string(),
        quarantine_table: "retail__trellara__fanin___trellara_quarantine".to_string(),
        verification_table: "retail__trellara__fanin___trellara_verification".to_string(),
        epoch_row: LakeRawCdcEpochRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            dataset_id: "retail".to_string(),
            state: LakeCompletenessState::Complete,
            policy: "wait_all_required".to_string(),
            opened_at: "planned_epoch_open".to_string(),
            sealed_at: "planned_epoch_seal".to_string(),
            required_source_count: 2,
            complete_source_count: 2,
            missing_source_count: 0,
            quarantined_source_count: 0,
            transaction_count: 3,
            change_count: 4,
            checksum_rollup: 99,
            manifest_digest: "abc123".to_string(),
            iceberg_snapshot_id: Some("snapshot-7".to_string()),
        },
        source_rows: vec![LakeRawCdcEpochSourceRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            source_id: "store-001".to_string(),
            state: LakeEpochSourceState::Complete,
            start_lsn: "0/16B6B00".to_string(),
            end_lsn: "0/16B6C50".to_string(),
            transaction_count: 3,
            change_count: 4,
            checksum_rollup: 99,
            lag_reason: None,
        }],
        table_rows: vec![LakeRawCdcEpochTableRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            relation: "public.sales".to_string(),
            transaction_count: 3,
            change_count: 4,
            checksum_rollup: 99,
        }],
        partition_rows: vec![LakeRawCdcEpochPartitionRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            source_id: "store-001".to_string(),
            partition_id: 7,
            first_commit_lsn: "0/16B6B00".to_string(),
            last_commit_lsn: "0/16B6C50".to_string(),
            transaction_count: 3,
            event_count: 4,
            checksum_rollup: 99,
        }],
        quarantine_rows: vec![LakeRawCdcEpochQuarantineRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            source_id: Some("store-002".to_string()),
            transaction_id: Some("tx-9".to_string()),
            commit_lsn: Some("0/16B6C90".to_string()),
            reason: "checksum_mismatch".to_string(),
            details: Some("source and lake checksums diverged".to_string()),
            recovery_command: Some("trellara lake retry --epoch epoch-2026-08-16T06".to_string()),
        }],
        verification_row: LakeRawCdcEpochVerificationRow {
            epoch_id: "epoch-2026-08-16T06".to_string(),
            verification_id: "verify:retail:epoch-2026-08-16T06:0000000000000063".to_string(),
            input_transaction_count: 3,
            input_change_count: 4,
            lake_transaction_count: 3,
            lake_change_count: 4,
            checksum_status: LakeEpochVerificationStatus::Match,
            completed_at: "planned_after_raw_cdc_files_durable".to_string(),
        },
    };

    let encoded = serde_json::to_string(&metadata).expect("serialize metadata artifact");
    let decoded: LakeRawCdcEpochMetadataPlan =
        serde_json::from_str(&encoded).expect("deserialize metadata artifact");

    assert_eq!(decoded, metadata);
}
