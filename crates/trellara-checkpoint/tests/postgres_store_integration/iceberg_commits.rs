use std::sync::Arc;

use super::support::*;
use trellara_checkpoint::{
    IcebergCommitStore, IcebergTableCommitIntent, IcebergTableCommitReceipt,
    IcebergTableCommitStatus,
};

#[tokio::test]
async fn postgres_store_persists_concurrent_iceberg_intents_and_receipts_idempotently(
) -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = Arc::new(connect_store(&database_url).await?);
    reset_store(&database_url).await?;
    let intent = intent();
    let mut tasks = Vec::new();
    for attempt in 0..16 {
        let store = store.clone();
        let mut intent = intent.clone();
        intent.planned_at = format!("2026-08-30T00:00:{attempt:02}Z");
        tasks.push(tokio::spawn(async move {
            store.record_iceberg_commit_intent(intent).await
        }));
    }
    for task in tasks {
        task.await??;
    }
    assert_eq!(
        store.load_iceberg_commit_intent(&intent.key()).await?,
        Some(intent.clone())
    );

    let receipt = receipt();
    let mut tasks = Vec::new();
    for attempt in 0..16 {
        let store = store.clone();
        let mut concurrent = receipt.clone();
        concurrent.committed_at = format!("2026-08-30T00:01:{attempt:02}Z");
        if attempt > 0 {
            concurrent.status = IcebergTableCommitStatus::AlreadyCommitted;
        }
        tasks.push(tokio::spawn(async move {
            store.record_iceberg_commit_receipt(concurrent).await
        }));
    }
    for task in tasks {
        task.await??;
    }
    let persisted = store
        .list_iceberg_commit_receipts_for_epoch(DATASET_ID, "epoch-1")
        .await?;
    assert_eq!(persisted.len(), 1);
    assert_eq!(
        persisted[0].snapshot_id, receipt.snapshot_id,
        "all concurrent observations must resolve to the same snapshot"
    );
    Ok(())
}

#[tokio::test]
async fn postgres_store_rejects_conflicting_iceberg_intent_replay() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = connect_store(&database_url).await?;
    reset_store(&database_url).await?;
    let original = intent();
    store.record_iceberg_commit_intent(original.clone()).await?;
    let mut conflicting = original;
    conflicting.record_count += 1;
    assert!(store
        .record_iceberg_commit_intent(conflicting)
        .await
        .is_err());
    Ok(())
}

fn intent() -> IcebergTableCommitIntent {
    IcebergTableCommitIntent {
        dataset_id: DATASET_ID.to_string(),
        epoch_id: "epoch-1".to_string(),
        epoch_commit_id: "a".repeat(64),
        lake_table_name: "orders_raw".to_string(),
        relation: "public.orders".to_string(),
        target: "analytics.retail.orders_cdc".to_string(),
        table_commit_id: "b".repeat(64),
        manifest_digest: "c".repeat(64),
        file_count: 1,
        record_count: 2,
        planned_at: "2026-08-30T00:00:00Z".to_string(),
    }
}

fn receipt() -> IcebergTableCommitReceipt {
    IcebergTableCommitReceipt {
        dataset_id: DATASET_ID.to_string(),
        epoch_id: "epoch-1".to_string(),
        epoch_commit_id: "a".repeat(64),
        target: "analytics.retail.orders_cdc".to_string(),
        table_commit_id: "b".repeat(64),
        snapshot_id: 101,
        file_count: 1,
        record_count: 2,
        status: IcebergTableCommitStatus::Committed,
        committed_at: "2026-08-30T00:00:01Z".to_string(),
    }
}
