use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use trellara_checkpoint::{IcebergCommitStore, InMemoryCheckpointStore};

use super::*;

#[tokio::test]
async fn checkpoint_commit_records_intent_before_catalog_callback() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let store = InMemoryCheckpointStore::new();
    let commit_count = Arc::new(AtomicUsize::new(0));

    let receipts = commit_iceberg_epoch_with_checkpoint(
        &store,
        &plan,
        "planned-at",
        "committed-at",
        |table| {
            let store = store.clone();
            let plan = plan.clone();
            let commit_count = Arc::clone(&commit_count);
            async move {
                let intent = iceberg_commit_intents_from_plan(&plan, "planned-at")
                    .into_iter()
                    .find(|intent| intent.table_commit_id == table.table_commit_id)
                    .expect("intent");
                assert!(store
                    .load_iceberg_commit_intent(&intent.key())
                    .await
                    .expect("load intent before commit")
                    .is_some());
                commit_count.fetch_add(1, Ordering::SeqCst);
                Ok(receipt(
                    &table,
                    100 + i64::try_from(table.file_count).expect("file count"),
                    IcebergTableCommitStatus::Committed,
                ))
            }
        },
    )
    .await
    .expect("commit with checkpoint");

    assert_eq!(receipts.len(), plan.tables.len());
    assert_eq!(commit_count.load(Ordering::SeqCst), plan.tables.len());
    for receipt in &receipts {
        let row = iceberg_commit_receipt_checkpoint_row(&plan, receipt, "committed-at")
            .expect("receipt row");
        assert!(store
            .load_iceberg_commit_receipt(&row.key())
            .await
            .expect("load receipt")
            .is_some());
    }
}

#[tokio::test]
async fn checkpoint_commit_skips_catalog_when_receipt_already_exists() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let store = InMemoryCheckpointStore::new();
    let existing = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    let existing_row = iceberg_commit_receipt_checkpoint_row(&plan, &existing, "committed-at")
        .expect("receipt row");
    store
        .record_iceberg_commit_receipt(existing_row)
        .await
        .expect("persist receipt");
    let commit_count = Arc::new(AtomicUsize::new(0));

    let receipts = commit_iceberg_epoch_with_checkpoint(
        &store,
        &plan,
        "planned-at",
        "committed-at",
        |table| {
            let commit_count = Arc::clone(&commit_count);
            async move {
                commit_count.fetch_add(1, Ordering::SeqCst);
                Ok(receipt(&table, 202, IcebergTableCommitStatus::Committed))
            }
        },
    )
    .await
    .expect("commit with checkpoint");

    assert_eq!(receipts.len(), plan.tables.len());
    assert_eq!(commit_count.load(Ordering::SeqCst), plan.tables.len() - 1);
    assert_eq!(
        receipts[0].status,
        IcebergTableCommitStatus::AlreadyCommitted
    );
    assert_eq!(receipts[0].snapshot_id, 101);
}

#[tokio::test]
async fn checkpoint_store_readiness_loads_epoch_receipts_from_checkpoint_store() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let store = InMemoryCheckpointStore::new();
    for (index, table) in plan.tables.iter().enumerate() {
        let receipt = receipt(
            table,
            i64::try_from(index + 1).expect("snapshot id"),
            IcebergTableCommitStatus::Committed,
        );
        store
            .record_iceberg_commit_receipt(
                iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
                    .expect("receipt row"),
            )
            .await
            .expect("record receipt");
    }

    let report = verify_iceberg_epoch_checkpoint_store_readiness(&store, &plan)
        .await
        .expect("readiness report");

    assert!(report.ready_for_epoch_metadata);
    assert_eq!(report.proven_table_count, plan.tables.len());
    assert!(report.missing_tables.is_empty());
    assert_eq!(report.snapshot_ids.len(), plan.tables.len());
}

#[tokio::test]
async fn checkpoint_store_readiness_keeps_epoch_metadata_blocked_for_partial_receipts() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let store = InMemoryCheckpointStore::new();
    let receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    store
        .record_iceberg_commit_receipt(
            iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
                .expect("receipt row"),
        )
        .await
        .expect("record receipt");

    let report = verify_iceberg_epoch_checkpoint_store_readiness(&store, &plan)
        .await
        .expect("readiness report");

    assert!(!report.ready_for_epoch_metadata);
    assert_eq!(report.proven_table_count, 1);
    assert_eq!(report.missing_tables.len(), plan.tables.len() - 1);
    assert!(report
        .tables
        .iter()
        .any(|table| table.status == IcebergTableReadinessStatus::MissingCheckpointReceipt));
}
