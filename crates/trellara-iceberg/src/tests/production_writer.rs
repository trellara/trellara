use std::collections::HashMap;

use apache_iceberg::memory::{MemoryCatalogBuilder, MEMORY_CATALOG_WAREHOUSE};
use apache_iceberg::CatalogBuilder;
use trellara_checkpoint::InMemoryCheckpointStore;
use trellara_lake::{raw_cdc_lake_ddl_ack_evidence, LakeRawCdcRowIntent, RawCdcLakeDdlAckRequest};

use super::*;

#[tokio::test]
async fn production_writer_publishes_completeness_last_and_replays_without_duplicates() {
    let catalog = MemoryCatalogBuilder::default()
        .load(
            "test",
            HashMap::from([(
                MEMORY_CATALOG_WAREHOUSE.to_string(),
                format!("memory:///warehouse-{}", uuid::Uuid::new_v4()),
            )]),
        )
        .await
        .expect("memory catalog");
    let object_store = super::object_store::FaultInjectingStore::default();
    let checkpoint = InMemoryCheckpointStore::new();
    let mut write_plan = raw_cdc_plan();
    write_plan.row_intents = raw_rows();
    let config = commit_config()
        .with_s3_object_store(
            IcebergS3ObjectStoreConfig::new("lake", "us-west-2").with_key_prefix("warehouse"),
        )
        .expect("object config");
    let raw_specs = plan_raw_cdc_iceberg_table_provisioning(&write_plan, &config)
        .expect("raw provisioning plans");
    let metadata_specs =
        plan_iceberg_metadata_table_specs(&write_plan, &config).expect("metadata specs");
    let evidence = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "database-a".to_string(),
        dataset_id: write_plan.dataset_id.clone(),
        barrier_id: "barrier-1".to_string(),
        ack_lsn: "0/16B6D00".to_string(),
        schema_version: "schema-v2".to_string(),
        epoch_id: write_plan.epoch_id.clone(),
        metadata_table: write_plan.epoch_metadata.epochs_table.clone(),
        partition_metadata_table: write_plan.epoch_metadata.epoch_partitions_table.clone(),
        manifest_digest: write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
    })
    .expect("lake DDL evidence");
    let ack = IcebergDdlAcknowledgement::from_raw_cdc_lake_ack(
        &evidence,
        raw_specs
            .iter()
            .map(|plan| plan.schema_fingerprint_sha256.clone())
            .chain(
                metadata_specs
                    .iter()
                    .map(|spec| spec.schema_fingerprint_sha256.clone()),
            ),
    )
    .expect("DDL acknowledgement");

    let first = write_production_iceberg_epoch(
        &catalog,
        &object_store,
        &checkpoint,
        &write_plan,
        &config,
        Some(&ack),
        ProductionIcebergCommitTimes::new("2026-08-30T00:00:00Z", "2026-08-30T00:00:01Z"),
    )
    .await
    .expect("initial production write");
    assert_eq!(first.raw_uploads.len(), 2);
    assert_eq!(first.raw_receipts.len(), 2);
    assert_eq!(first.supporting_metadata_receipts.len(), 1);
    assert_eq!(first.completeness_receipts.len(), 1);
    assert_eq!(
        first.metadata_commit_bundle.completeness_commit_plan.tables[0]
            .target
            .name,
        "_trellara_epochs"
    );
    assert!(first
        .raw_receipts
        .iter()
        .chain(&first.supporting_metadata_receipts)
        .chain(&first.completeness_receipts)
        .all(|receipt| receipt.status == IcebergTableCommitStatus::Committed));

    let replay = write_production_iceberg_epoch(
        &catalog,
        &object_store,
        &checkpoint,
        &write_plan,
        &config,
        None,
        ProductionIcebergCommitTimes::new("2026-08-30T01:00:00Z", "2026-08-30T01:00:01Z"),
    )
    .await
    .expect("retry production write");
    assert!(replay
        .raw_uploads
        .iter()
        .all(|upload| upload.upload.status == IcebergImmutableUploadStatus::AlreadyPresent));
    assert!(replay
        .metadata_uploads
        .iter()
        .all(|upload| upload.upload.status == IcebergImmutableUploadStatus::AlreadyPresent));
    assert!(replay
        .raw_receipts
        .iter()
        .chain(&replay.supporting_metadata_receipts)
        .chain(&replay.completeness_receipts)
        .all(|receipt| receipt.status == IcebergTableCommitStatus::AlreadyCommitted));
    assert_eq!(
        first
            .raw_receipts
            .iter()
            .map(|receipt| receipt.snapshot_id)
            .collect::<Vec<_>>(),
        replay
            .raw_receipts
            .iter()
            .map(|receipt| receipt.snapshot_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        first.completeness_receipts[0].snapshot_id,
        replay.completeness_receipts[0].snapshot_id
    );
}

fn raw_rows() -> Vec<LakeRawCdcRowIntent> {
    vec![
        row(
            "source-a",
            0,
            "public.orders",
            "tx-orders",
            "0/16B6C50",
            1,
            "insert",
            "order-1",
            "event-orders-1",
            3,
        ),
        row(
            "source-a",
            0,
            "public.orders",
            "tx-orders",
            "0/16B6C50",
            2,
            "update",
            "order-1",
            "event-orders-2",
            3,
        ),
        row(
            "source-b",
            1,
            "public.customers",
            "tx-customers",
            "0/16B6D00",
            1,
            "insert",
            "customer-1",
            "event-customers-1",
            4,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn row(
    source_id: &str,
    source_bucket: u32,
    relation: &str,
    transaction_id: &str,
    commit_lsn: &str,
    total_order: u32,
    operation: &str,
    record_key: &str,
    idempotency_key: &str,
    envelope_checksum: u64,
) -> LakeRawCdcRowIntent {
    LakeRawCdcRowIntent {
        source_id: source_id.to_string(),
        source_bucket,
        database_id: "database-a".to_string(),
        dataset_id: "retail".to_string(),
        relation: relation.to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B0000".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_788_048_000_000,
        total_order,
        operation: operation.to_string(),
        record_key: Some(record_key.to_string()),
        idempotency_key: idempotency_key.to_string(),
        schema_fingerprint: None,
        schema_version: None,
        ddl_barrier_id: None,
        ddl_release_gate: None,
        ddl_schema_fingerprint_before: None,
        ddl_schema_fingerprint_after: None,
        envelope_checksum,
        manifest_id: None,
        manifest_boundary_mode: None,
        manifest_global_event_count: None,
        manifest_participating_partition_count: None,
        partition_key: None,
        epoch_id: "epoch-1".to_string(),
        ingested_at: "2026-08-30T00:00:00Z".to_string(),
        payload_before: Vec::new(),
        payload_after: Vec::new(),
    }
}
