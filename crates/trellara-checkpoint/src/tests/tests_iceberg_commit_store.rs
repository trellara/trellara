use super::*;

#[tokio::test]
async fn iceberg_commit_store_records_intent_and_receipt_idempotently() {
    let store = InMemoryCheckpointStore::new();
    let intent = intent();
    let receipt = receipt();
    let key = intent.key();

    store
        .record_iceberg_commit_intent(intent.clone())
        .await
        .expect("record intent");
    store
        .record_iceberg_commit_intent(intent.clone())
        .await
        .expect("idempotent intent replay");
    store
        .record_iceberg_commit_receipt(receipt.clone())
        .await
        .expect("record receipt");
    store
        .record_iceberg_commit_receipt(receipt.clone())
        .await
        .expect("idempotent receipt replay");

    assert_eq!(
        store
            .load_iceberg_commit_intent(&key)
            .await
            .expect("load intent"),
        Some(intent)
    );
    assert_eq!(
        store
            .load_iceberg_commit_receipt(&key)
            .await
            .expect("load receipt"),
        Some(receipt)
    );
}

#[tokio::test]
async fn iceberg_commit_store_accepts_concurrent_attempt_metadata_for_same_commit() {
    let store = InMemoryCheckpointStore::new();
    let first_intent = intent();
    let mut replay_intent = first_intent.clone();
    replay_intent.planned_at = "later retry".to_string();
    store
        .record_iceberg_commit_intent(first_intent)
        .await
        .expect("record first intent");
    store
        .record_iceberg_commit_intent(replay_intent)
        .await
        .expect("same commit may resume with a new attempt timestamp");

    let first_receipt = receipt();
    let mut reconciled_receipt = first_receipt.clone();
    reconciled_receipt.status = IcebergTableCommitStatus::AlreadyCommitted;
    reconciled_receipt.committed_at = "later reconciliation".to_string();
    store
        .record_iceberg_commit_receipt(first_receipt)
        .await
        .expect("record direct receipt");
    store
        .record_iceberg_commit_receipt(reconciled_receipt)
        .await
        .expect("same snapshot may be observed through reconciliation");
}

#[tokio::test]
async fn iceberg_commit_store_rejects_conflicting_intent_replay() {
    let store = InMemoryCheckpointStore::new();
    let mut conflicting = intent();
    store
        .record_iceberg_commit_intent(conflicting.clone())
        .await
        .expect("record intent");

    conflicting.record_count += 1;
    let error = store
        .record_iceberg_commit_intent(conflicting)
        .await
        .expect_err("conflict");

    assert!(error
        .to_string()
        .contains("conflicting Iceberg commit intent"));
}

#[tokio::test]
async fn iceberg_commit_store_rejects_conflicting_receipt_replay() {
    let store = InMemoryCheckpointStore::new();
    let mut conflicting = receipt();
    store
        .record_iceberg_commit_receipt(conflicting.clone())
        .await
        .expect("record receipt");

    conflicting.snapshot_id += 1;
    let error = store
        .record_iceberg_commit_receipt(conflicting)
        .await
        .expect_err("conflict");

    assert!(error
        .to_string()
        .contains("conflicting Iceberg commit receipt"));
}

#[tokio::test]
async fn iceberg_commit_store_lists_receipts_for_epoch_deterministically() {
    let store = InMemoryCheckpointStore::new();
    let mut second = receipt();
    second.target = "analytics.retail.z_sales_cdc".to_string();
    second.table_commit_id = digest('d');
    second.snapshot_id = 43;
    let mut other_epoch = receipt();
    other_epoch.epoch_id = "epoch-2".to_string();
    other_epoch.table_commit_id = digest('e');
    other_epoch.snapshot_id = 44;

    store
        .record_iceberg_commit_receipt(second.clone())
        .await
        .expect("record second receipt");
    store
        .record_iceberg_commit_receipt(other_epoch)
        .await
        .expect("record other epoch receipt");
    store
        .record_iceberg_commit_receipt(receipt())
        .await
        .expect("record first receipt");

    let receipts = store
        .list_iceberg_commit_receipts_for_epoch("retail", "epoch-1")
        .await
        .expect("list receipts");

    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0].target, "analytics.retail.sales_cdc");
    assert_eq!(receipts[1].target, second.target);
}

#[tokio::test]
async fn iceberg_commit_store_rejects_invalid_epoch_receipt_lookup() {
    let store = InMemoryCheckpointStore::new();

    let error = store
        .list_iceberg_commit_receipts_for_epoch(" retail", "epoch-1")
        .await
        .expect_err("invalid lookup");

    assert!(error
        .to_string()
        .contains("invalid Iceberg commit dataset_id"));
}

#[tokio::test]
async fn iceberg_commit_store_rejects_invalid_commit_identity() {
    let store = InMemoryCheckpointStore::new();
    let mut intent = intent();
    intent.table_commit_id = "short".to_string();

    let error = store
        .record_iceberg_commit_intent(intent)
        .await
        .expect_err("invalid");

    assert!(error
        .to_string()
        .contains("invalid Iceberg commit table_commit_id"));
}

fn intent() -> IcebergTableCommitIntent {
    IcebergTableCommitIntent {
        dataset_id: "retail".to_string(),
        epoch_id: "epoch-1".to_string(),
        epoch_commit_id: digest('a'),
        lake_table_name: "retail__public__sales__raw_cdc".to_string(),
        relation: "public.sales".to_string(),
        target: "analytics.retail.sales_cdc".to_string(),
        table_commit_id: digest('b'),
        manifest_digest: digest('c'),
        file_count: 2,
        record_count: 10,
        planned_at: "planned".to_string(),
    }
}

fn receipt() -> IcebergTableCommitReceipt {
    IcebergTableCommitReceipt {
        dataset_id: "retail".to_string(),
        epoch_id: "epoch-1".to_string(),
        epoch_commit_id: digest('a'),
        target: "analytics.retail.sales_cdc".to_string(),
        table_commit_id: digest('b'),
        snapshot_id: 42,
        file_count: 2,
        record_count: 10,
        status: IcebergTableCommitStatus::Committed,
        committed_at: "committed".to_string(),
    }
}

fn digest(character: char) -> String {
    character.to_string().repeat(64)
}
