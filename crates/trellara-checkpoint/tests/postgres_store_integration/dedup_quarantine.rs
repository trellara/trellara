use super::support::*;
use trellara_checkpoint::{ApplyDecision, DedupStore, TransactionKey};

#[tokio::test]
async fn postgres_store_persists_dedup_and_quarantine_state() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = connect_store(&database_url).await?;
    reset_store(&database_url).await?;
    let flow = flow_key();

    let transaction = TransactionKey {
        source_id: SOURCE_ID.to_string(),
        database_id: DATABASE_ID.to_string(),
        dataset_id: DATASET_ID.to_string(),
        transaction_id: "tx-checkpoint-1".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
    };
    assert_eq!(
        store.apply_decision(&transaction).await?,
        ApplyDecision::Apply
    );
    store.record_applied(transaction.clone()).await?;
    assert_eq!(
        store.apply_decision(&transaction).await?,
        ApplyDecision::SkipDuplicate
    );
    assert!(store.clear_applied_transaction(&transaction).await?);
    assert_eq!(
        store.apply_decision(&transaction).await?,
        ApplyDecision::Apply
    );
    assert!(!store.clear_applied_transaction(&transaction).await?);

    insert_quarantine(&database_url, "tx-checkpoint-quarantine", "0/16B7000").await?;
    let quarantine = store
        .load_latest_quarantine(&flow)
        .await?
        .expect("latest quarantine");
    assert_eq!(quarantine.transaction_id, "tx-checkpoint-quarantine");
    assert_eq!(quarantine.commit_lsn, "0/16B7000");
    assert_eq!(quarantine.reason, "target_postgres_error");
    assert_eq!(quarantine.attempt_count, 1);
    let quarantine_key = TransactionKey {
        source_id: SOURCE_ID.to_string(),
        database_id: DATABASE_ID.to_string(),
        dataset_id: DATASET_ID.to_string(),
        transaction_id: "tx-checkpoint-quarantine".to_string(),
        commit_lsn: "0/16B7000".to_string(),
    };
    let exact_quarantine = store
        .load_quarantine(&quarantine_key)
        .await?
        .expect("exact quarantine");
    assert_eq!(exact_quarantine.transaction_id, "tx-checkpoint-quarantine");
    assert_eq!(exact_quarantine.detail, "target table is missing");
    let quarantines = store.list_quarantine(&flow, 10).await?;
    assert_eq!(quarantines.len(), 1);
    assert_eq!(quarantines[0].transaction_id, "tx-checkpoint-quarantine");
    assert!(store.clear_quarantine(&quarantine_key).await?);
    assert!(store.load_latest_quarantine(&flow).await?.is_none());
    assert!(store.load_quarantine(&quarantine_key).await?.is_none());
    assert!(
        !store
            .clear_quarantine(&TransactionKey {
                source_id: SOURCE_ID.to_string(),
                database_id: DATABASE_ID.to_string(),
                dataset_id: DATASET_ID.to_string(),
                transaction_id: "tx-checkpoint-quarantine".to_string(),
                commit_lsn: "0/16B7000".to_string(),
            })
            .await?
    );

    Ok(())
}
