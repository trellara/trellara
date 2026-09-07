use std::collections::BTreeMap;

use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use trellara_lake::{
    LakeEpochSourceState, LakeRawCdcEpochPartitionRow, LakeRawCdcEpochQuarantineRow,
    LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow,
};

use super::*;

#[test]
fn metadata_writer_encodes_every_evidence_relation_as_parquet() {
    let mut plan = raw_cdc_plan();
    plan.epoch_metadata.source_rows = vec![LakeRawCdcEpochSourceRow {
        epoch_id: "epoch-1".to_string(),
        source_id: "source-a".to_string(),
        state: LakeEpochSourceState::Complete,
        start_lsn: "0/16B0000".to_string(),
        end_lsn: "0/16B6C50".to_string(),
        transaction_count: 2,
        change_count: 3,
        checksum_rollup: 7,
        lag_reason: None,
    }];
    plan.epoch_metadata.table_rows = vec![LakeRawCdcEpochTableRow {
        epoch_id: "epoch-1".to_string(),
        relation: "public.orders".to_string(),
        transaction_count: 2,
        change_count: 3,
        checksum_rollup: 7,
    }];
    plan.epoch_metadata.partition_rows = vec![LakeRawCdcEpochPartitionRow {
        epoch_id: "epoch-1".to_string(),
        source_id: "source-a".to_string(),
        partition_id: 0,
        first_commit_lsn: "0/16B0000".to_string(),
        last_commit_lsn: "0/16B6C50".to_string(),
        transaction_count: 2,
        event_count: 3,
        checksum_rollup: 7,
    }];
    plan.epoch_metadata.quarantine_rows = vec![LakeRawCdcEpochQuarantineRow {
        epoch_id: "epoch-1".to_string(),
        source_id: Some("source-b".to_string()),
        transaction_id: None,
        commit_lsn: None,
        reason: "source_lag".to_string(),
        details: Some("test evidence".to_string()),
        recovery_command: Some("trellara lake replay".to_string()),
    }];
    let specs = plan_iceberg_metadata_table_specs(&plan, &commit_config()).expect("specs");
    let snapshots = BTreeMap::from([
        ("analytics.retail.customers_cdc".to_string(), 101),
        ("analytics.retail.orders_cdc".to_string(), 102),
    ]);

    let files =
        encode_iceberg_metadata_files(&plan, &specs, "trellara-iceberg-epoch:commit-1", &snapshots)
            .expect("encode metadata");

    assert_eq!(files.len(), 6);
    assert_eq!(
        files.iter().map(|file| file.kind).collect::<Vec<_>>(),
        vec![
            IcebergMetadataTableKind::Source,
            IcebergMetadataTableKind::Table,
            IcebergMetadataTableKind::Partition,
            IcebergMetadataTableKind::Quarantine,
            IcebergMetadataTableKind::Verification,
            IcebergMetadataTableKind::Completeness,
        ]
    );
    for file in files {
        assert!(file.planned_object_key.ends_with("part-00000.parquet"));
        let mut reader =
            ParquetRecordBatchReader::try_new(file.content, 1024).expect("open metadata parquet");
        let batch = reader.next().expect("batch").expect("read batch");
        assert_eq!(batch.num_rows() as u64, file.record_count);
        assert_eq!(batch.schema().field(0).metadata()["PARQUET:field_id"], "1");
    }
}

#[tokio::test]
async fn uploaded_metadata_evidence_uses_verified_object_identity() {
    let plan = raw_cdc_plan();
    let specs = plan_iceberg_metadata_table_specs(&plan, &commit_config()).expect("specs");
    let files = encode_iceberg_metadata_files(
        &plan,
        &specs,
        "trellara-iceberg-epoch:commit-1",
        &BTreeMap::from([("analytics.retail.orders_cdc".to_string(), 101)]),
    )
    .expect("encode metadata");
    let encoded = files
        .into_iter()
        .find(|file| file.kind == IcebergMetadataTableKind::Completeness)
        .expect("completeness file");
    let store = super::object_store::FaultInjectingStore::default();
    let config = IcebergS3ObjectStoreConfig::new("lake", "us-west-2").with_key_prefix("warehouse");

    let uploaded = upload_iceberg_metadata_file(&store, &config, encoded)
        .await
        .expect("upload metadata");
    assert_eq!(
        uploaded.completed_file.file_size_in_bytes,
        uploaded.upload.content_length
    );
    assert_eq!(
        uploaded.completed_file.object_version.as_deref(),
        Some("v1")
    );
    assert!(uploaded
        .completed_file
        .file_uri
        .starts_with("s3://lake/warehouse/"));
}

#[tokio::test]
async fn metadata_commit_bundle_keeps_completeness_as_the_final_catalog_commit() {
    let plan = raw_cdc_plan();
    let config = commit_config();
    let raw_commit =
        plan_iceberg_epoch_commit(&plan, &config, completed_files()).expect("raw commit plan");
    let specs = plan_iceberg_metadata_table_specs(&plan, &config).expect("specs");
    let snapshots = BTreeMap::from([
        ("analytics.retail.customers_cdc".to_string(), 101),
        ("analytics.retail.orders_cdc".to_string(), 102),
    ]);
    let encoded =
        encode_iceberg_metadata_files(&plan, &specs, "trellara-iceberg-epoch:commit-1", &snapshots)
            .expect("encode metadata");
    let store = super::object_store::FaultInjectingStore::default();
    let object_config = IcebergS3ObjectStoreConfig::new("lake", "us-west-2");
    let mut uploaded = Vec::new();
    for file in encoded {
        uploaded.push(
            upload_iceberg_metadata_file(&store, &object_config, file)
                .await
                .expect("upload"),
        );
    }

    let bundle = plan_iceberg_metadata_commit_bundle(
        &plan,
        &raw_commit,
        &specs,
        snapshots,
        "trellara-iceberg-epoch:commit-1",
        uploaded,
    )
    .expect("metadata commit bundle");

    assert!(bundle
        .supporting_metadata_commit_plan
        .tables
        .iter()
        .all(|table| table.target.name != "_trellara_epochs"));
    assert_eq!(bundle.completeness_commit_plan.tables.len(), 1);
    assert_eq!(
        bundle.completeness_commit_plan.tables[0].target.name,
        "_trellara_epochs"
    );
    assert!(bundle
        .visibility_rule
        .ends_with("_trellara_epochs release marker"));
}
