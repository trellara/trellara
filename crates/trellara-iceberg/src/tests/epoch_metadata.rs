use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use trellara_checkpoint::{IcebergCommitStore, InMemoryCheckpointStore};

use super::*;

#[test]
fn epoch_metadata_table_spec_is_the_l1_completeness_table() {
    let spec =
        plan_iceberg_epoch_metadata_table_spec(&raw_cdc_plan(), &commit_config()).expect("spec");

    assert_eq!(spec.lake_table_name, "epochs");
    assert_eq!(spec.relation, ICEBERG_EPOCH_METADATA_RELATION);
    assert_eq!(
        spec.target.qualified_name(),
        "analytics.retail._trellara_epochs"
    );
    assert_eq!(spec.schema_fingerprint_sha256.len(), 64);
    assert_eq!(spec.partition_fields, Vec::new());
    assert_eq!(spec.write_mode, "append_only_epoch_completeness");
    assert!(spec
        .visibility_rule
        .contains("after every raw changelog table receipt"));
    assert!(spec
        .columns
        .iter()
        .any(|column| column.name == "iceberg_snapshot_id" && column.required));
    assert!(spec
        .columns
        .iter()
        .any(|column| column.name == "raw_table_snapshot_ids_json" && column.required));
}

#[test]
fn fanin_l1_table_specs_package_changelog_and_epochs_table() {
    let specs =
        plan_iceberg_fanin_l1_table_specs(&raw_cdc_plan(), &commit_config()).expect("specs");

    assert_eq!(specs.dataset_id, "retail");
    assert_eq!(specs.epoch_id, "epoch-1");
    assert_eq!(
        specs.completeness_contract,
        "append_only_raw_changelog_plus_queryable_trellara_epochs"
    );
    assert_eq!(specs.raw_changelog_tables.len(), 2);
    assert!(specs
        .raw_changelog_tables
        .iter()
        .all(|table| table.partition_fields.len() == 2));
    assert_eq!(
        specs.epoch_metadata_table.target.qualified_name(),
        "analytics.retail._trellara_epochs"
    );
}

#[test]
fn plans_epoch_metadata_append_only_after_raw_receipts_are_complete() {
    let raw_write_plan = raw_cdc_plan();
    let raw_commit_plan =
        plan_iceberg_epoch_commit(&raw_write_plan, &commit_config(), completed_files())
            .expect("raw commit plan");
    let receipts = raw_receipts(&raw_commit_plan);
    let metadata_file = metadata_file(&raw_write_plan, &raw_commit_plan, &receipts);

    let metadata_append = plan_iceberg_epoch_metadata_append(
        &raw_write_plan,
        &raw_commit_plan,
        &commit_config(),
        &receipts,
        metadata_file,
    )
    .expect("metadata append plan");

    assert_eq!(metadata_append.dataset_id, raw_commit_plan.dataset_id);
    assert_eq!(metadata_append.epoch_id, raw_commit_plan.epoch_id);
    assert_eq!(
        metadata_append.epoch_commit_id,
        raw_commit_plan.epoch_commit_id
    );
    assert_eq!(
        metadata_append.metadata_table,
        "analytics.retail._trellara_epochs"
    );
    assert_eq!(metadata_append.raw_snapshot_ids, snapshot_map(&receipts));
    assert_eq!(
        metadata_append.epoch_snapshot_reference,
        format!("trellara-iceberg-epoch:{}", raw_commit_plan.epoch_commit_id)
    );
    assert_eq!(metadata_append.metadata_commit_plan.file_count, 1);
    assert_eq!(metadata_append.metadata_commit_plan.record_count, 1);
    assert_eq!(metadata_append.metadata_commit_plan.tables.len(), 1);
    let table = &metadata_append.metadata_commit_plan.tables[0];
    assert_eq!(table.lake_table_name, "epochs");
    assert_eq!(table.relation, ICEBERG_EPOCH_METADATA_RELATION);
    assert_eq!(table.file_count, 1);
    assert_eq!(table.record_count, 1);
    assert_eq!(
        table
            .snapshot_properties
            .get(SNAPSHOT_PROPERTY_METADATA_KIND),
        Some(&ICEBERG_EPOCH_METADATA_KIND.to_string())
    );
    assert_eq!(
        table
            .snapshot_properties
            .get(SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS),
        Some(&"analytics.retail.customers_cdc=202,analytics.retail.orders_cdc=101".to_string())
    );
}

#[test]
fn blocks_epoch_metadata_append_when_raw_receipts_are_partial() {
    let raw_write_plan = raw_cdc_plan();
    let raw_commit_plan =
        plan_iceberg_epoch_commit(&raw_write_plan, &commit_config(), completed_files())
            .expect("raw commit plan");
    let receipts = vec![receipt(
        &raw_commit_plan.tables[0],
        101,
        IcebergTableCommitStatus::Committed,
    )];
    let metadata_file = metadata_file(&raw_write_plan, &raw_commit_plan, &receipts);

    let error = plan_iceberg_epoch_metadata_append(
        &raw_write_plan,
        &raw_commit_plan,
        &commit_config(),
        &receipts,
        metadata_file,
    )
    .expect_err("metadata must wait for all raw receipts");

    assert!(matches!(
        error,
        IcebergIntegrationError::EpochMetadataNotReady {
            missing_tables,
            ..
        } if missing_tables == vec!["analytics.retail.orders_cdc".to_string()]
            || missing_tables == vec!["analytics.retail.customers_cdc".to_string()]
    ));
}

#[test]
fn rejects_epoch_metadata_file_with_stale_snapshot_map() {
    let raw_write_plan = raw_cdc_plan();
    let raw_commit_plan =
        plan_iceberg_epoch_commit(&raw_write_plan, &commit_config(), completed_files())
            .expect("raw commit plan");
    let receipts = raw_receipts(&raw_commit_plan);
    let mut metadata_file = metadata_file(&raw_write_plan, &raw_commit_plan, &receipts);
    metadata_file
        .raw_table_snapshot_ids
        .insert("analytics.retail.orders_cdc".to_string(), 999);

    let error = plan_iceberg_epoch_metadata_append(
        &raw_write_plan,
        &raw_commit_plan,
        &commit_config(),
        &receipts,
        metadata_file,
    )
    .expect_err("stale metadata file should fail closed");

    assert!(matches!(
        error,
        IcebergIntegrationError::CompletedDataFileMismatch {
            field: "raw_table_snapshot_ids",
            ..
        }
    ));
}

#[tokio::test]
async fn metadata_append_reuses_checkpoint_commit_path() {
    let raw_write_plan = raw_cdc_plan();
    let raw_commit_plan =
        plan_iceberg_epoch_commit(&raw_write_plan, &commit_config(), completed_files())
            .expect("raw commit plan");
    let receipts = raw_receipts(&raw_commit_plan);
    let metadata_file = metadata_file(&raw_write_plan, &raw_commit_plan, &receipts);
    let metadata_append = plan_iceberg_epoch_metadata_append(
        &raw_write_plan,
        &raw_commit_plan,
        &commit_config(),
        &receipts,
        metadata_file,
    )
    .expect("metadata append plan");
    let store = InMemoryCheckpointStore::new();
    let commit_count = Arc::new(AtomicUsize::new(0));

    let metadata_receipts = commit_iceberg_epoch_with_checkpoint(
        &store,
        &metadata_append.metadata_commit_plan,
        "planned-metadata-at",
        "committed-metadata-at",
        |table| {
            let commit_count = Arc::clone(&commit_count);
            async move {
                commit_count.fetch_add(1, Ordering::SeqCst);
                Ok(receipt(&table, 303, IcebergTableCommitStatus::Committed))
            }
        },
    )
    .await
    .expect("metadata checkpoint commit");

    assert_eq!(metadata_receipts.len(), 1);
    assert_eq!(commit_count.load(Ordering::SeqCst), 1);
    let checkpoint_rows = store
        .list_iceberg_commit_receipts_for_epoch("retail", "epoch-1")
        .await
        .expect("checkpoint rows");
    assert_eq!(checkpoint_rows.len(), 1);
    assert_eq!(
        checkpoint_rows[0].target,
        "analytics.retail._trellara_epochs"
    );
}

fn raw_receipts(plan: &IcebergEpochCommitPlan) -> Vec<IcebergTableCommitReceipt> {
    plan.tables
        .iter()
        .map(|table| {
            let snapshot_id = match table.lake_table_name.as_str() {
                "retail_raw_orders" => 101,
                "retail_raw_customers" => 202,
                other => panic!("unexpected raw table {other}"),
            };
            receipt(table, snapshot_id, IcebergTableCommitStatus::Committed)
        })
        .collect()
}

fn metadata_file(
    write_plan: &trellara_lake::LakeRawCdcEpochWritePlan,
    commit_plan: &IcebergEpochCommitPlan,
    receipts: &[IcebergTableCommitReceipt],
) -> trellara_lake::LakeRawCdcEpochMetadataParquetWriteEvidence {
    let snapshot_ids = snapshot_map(receipts);
    let output = trellara_lake::write_raw_cdc_epoch_metadata_parquet_file(
        Vec::new(),
        write_plan,
        format!("trellara-iceberg-epoch:{}", commit_plan.epoch_commit_id),
        snapshot_ids,
        "s3://lake/retail/_trellara_epochs/epoch_id=epoch-1/part-00000.parquet",
        Some("metadata-version-1".to_string()),
    )
    .expect("metadata parquet evidence");
    output.evidence
}

fn snapshot_map(receipts: &[IcebergTableCommitReceipt]) -> BTreeMap<String, i64> {
    receipts
        .iter()
        .map(|receipt| (receipt.target.qualified_name(), receipt.snapshot_id))
        .collect()
}
