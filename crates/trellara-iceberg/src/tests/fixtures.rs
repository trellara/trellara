use trellara_lake::{
    LakeCompletenessState, LakeEpochVerificationStatus, LakeRawCdcCommitterTopology,
    LakeRawCdcDataFilePlan, LakeRawCdcDuplicateReplayEvidence, LakeRawCdcEpochMetadataPlan,
    LakeRawCdcEpochRow, LakeRawCdcEpochVerificationRow, LakeRawCdcEpochWritePlan,
};

use super::*;

pub(crate) const MANIFEST_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub(crate) fn receipt(
    table: &IcebergTableAppendPlan,
    snapshot_id: i64,
    status: IcebergTableCommitStatus,
) -> IcebergTableCommitReceipt {
    IcebergTableCommitReceipt {
        target: table.target.clone(),
        epoch_commit_id: table.epoch_commit_id.clone(),
        table_commit_id: table.table_commit_id.clone(),
        snapshot_id,
        file_count: table.file_count,
        record_count: table.record_count,
        status,
    }
}

pub(crate) fn commit_config() -> IcebergCommitConfig {
    IcebergCommitConfig::new(vec![
        IcebergTableMapping::new(
            "retail_raw_orders",
            IcebergTableIdentifier::new(["analytics", "retail"], "orders_cdc").expect("target"),
        ),
        IcebergTableMapping::new(
            "retail_raw_customers",
            IcebergTableIdentifier::new(["analytics", "retail"], "customers_cdc").expect("target"),
        ),
        IcebergTableMapping::new(
            "epochs",
            IcebergTableIdentifier::new(["analytics", "retail"], "_trellara_epochs")
                .expect("target"),
        ),
    ])
}

pub(crate) fn completed_files() -> Vec<IcebergCompletedDataFile> {
    vec![
        IcebergCompletedDataFile {
            planned_object_key: "epoch-1/retail_raw_orders/source_bucket=0/data.parquet"
                .to_string(),
            table_name: "retail_raw_orders".to_string(),
            relation: "public.orders".to_string(),
            source_bucket: 0,
            file_uri: "s3://lake/epoch-1/retail_raw_orders/source_bucket=0/data.parquet"
                .to_string(),
            file_format: IcebergFileFormat::Parquet,
            file_size_in_bytes: 1_024,
            content_sha256: "b".repeat(64),
            object_version: Some("version-1".to_string()),
            record_count: 2,
            checksum_rollup: 3,
        },
        IcebergCompletedDataFile {
            planned_object_key: "epoch-1/retail_raw_customers/source_bucket=1/data.parquet"
                .to_string(),
            table_name: "retail_raw_customers".to_string(),
            relation: "public.customers".to_string(),
            source_bucket: 1,
            file_uri: "s3://lake/epoch-1/retail_raw_customers/source_bucket=1/data.parquet"
                .to_string(),
            file_format: IcebergFileFormat::Parquet,
            file_size_in_bytes: 512,
            content_sha256: "c".repeat(64),
            object_version: Some("version-2".to_string()),
            record_count: 1,
            checksum_rollup: 4,
        },
    ]
}

pub(crate) fn raw_cdc_plan() -> LakeRawCdcEpochWritePlan {
    let data_files = vec![
        LakeRawCdcDataFilePlan {
            table_name: "retail_raw_orders".to_string(),
            relation: "public.orders".to_string(),
            source_bucket: 0,
            source_ids: vec!["source-a".to_string()],
            transaction_count: 1,
            change_count: 2,
            min_commit_lsn: "0/16B6C50".to_string(),
            max_commit_lsn: "0/16B6C50".to_string(),
            checksum_rollup: 3,
            idempotency_key_count: 2,
            object_key_hint: "epoch-1/retail_raw_orders/source_bucket=0/data.parquet".to_string(),
        },
        LakeRawCdcDataFilePlan {
            table_name: "retail_raw_customers".to_string(),
            relation: "public.customers".to_string(),
            source_bucket: 1,
            source_ids: vec!["source-b".to_string()],
            transaction_count: 1,
            change_count: 1,
            min_commit_lsn: "0/16B6D00".to_string(),
            max_commit_lsn: "0/16B6D00".to_string(),
            checksum_rollup: 4,
            idempotency_key_count: 1,
            object_key_hint: "epoch-1/retail_raw_customers/source_bucket=1/data.parquet"
                .to_string(),
        },
    ];
    LakeRawCdcEpochWritePlan {
        dataset_id: "retail".to_string(),
        epoch_id: "epoch-1".to_string(),
        transaction_count: 2,
        change_count: 3,
        duplicate_transaction_count: 0,
        skipped_dataset_transaction_count: 0,
        data_file_count: data_files.len(),
        checksum_rollup: 7,
        visibility_rule: "files before metadata".to_string(),
        duplicate_replay_evidence: LakeRawCdcDuplicateReplayEvidence {
            contract: "duplicate replay is skipped by transaction identity".to_string(),
            duplicate_transaction_count: 0,
            unique_transaction_count: 2,
            row_intent_count: 0,
            idempotency_key_count: 0,
            replay_safe: true,
        },
        committer_topology: LakeRawCdcCommitterTopology {
            strategy: "single_table_committer".to_string(),
            committer_count: 2,
            table_count: 2,
            source_bucket_count: 2,
            table_committers: Vec::new(),
            source_ack_boundary: "durable stream".to_string(),
            catalog_backpressure_rule: "consumer backpressure".to_string(),
        },
        commit_steps: Vec::new(),
        recovery_scenarios: Vec::new(),
        data_files,
        row_intents: Vec::new(),
        epoch_metadata: LakeRawCdcEpochMetadataPlan {
            epochs_table: "epochs".to_string(),
            epoch_sources_table: "epoch_sources".to_string(),
            epoch_tables_table: "epoch_tables".to_string(),
            epoch_partitions_table: "epoch_partitions".to_string(),
            quarantine_table: "quarantine".to_string(),
            verification_table: "verification".to_string(),
            epoch_row: LakeRawCdcEpochRow {
                epoch_id: "epoch-1".to_string(),
                dataset_id: "retail".to_string(),
                state: LakeCompletenessState::Complete,
                policy: "wait_all_required".to_string(),
                opened_at: "open".to_string(),
                sealed_at: "sealed".to_string(),
                required_source_count: 2,
                complete_source_count: 2,
                missing_source_count: 0,
                quarantined_source_count: 0,
                transaction_count: 2,
                change_count: 3,
                checksum_rollup: 7,
                manifest_digest: MANIFEST_DIGEST.to_string(),
                iceberg_snapshot_id: None,
            },
            source_rows: Vec::new(),
            table_rows: Vec::new(),
            partition_rows: Vec::new(),
            quarantine_rows: Vec::new(),
            verification_row: LakeRawCdcEpochVerificationRow {
                epoch_id: "epoch-1".to_string(),
                verification_id: "verify:retail:epoch-1:0000000000000007".to_string(),
                input_transaction_count: 2,
                input_change_count: 3,
                lake_transaction_count: 2,
                lake_change_count: 3,
                checksum_status: LakeEpochVerificationStatus::Match,
                completed_at: "complete".to_string(),
            },
        },
    }
}
