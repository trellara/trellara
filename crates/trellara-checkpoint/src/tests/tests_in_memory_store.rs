use super::*;
use proptest::prelude::*;
use std::collections::HashSet as StdHashSet;
use trellara_protocol::Checkpoint;
#[tokio::test]
async fn duplicate_transactions_are_skipped_after_recording() {
    let store = InMemoryCheckpointStore::new();
    let envelope = sample_envelope();
    let key = TransactionKey::from_envelope(&envelope);

    assert_eq!(
        store.apply_decision(&key).await.unwrap(),
        ApplyDecision::Apply
    );
    store.record_applied(key.clone()).await.unwrap();
    assert_eq!(
        store.apply_decision(&key).await.unwrap(),
        ApplyDecision::SkipDuplicate
    );
}

#[tokio::test]
async fn dedup_store_rejects_empty_transaction_identity() {
    let store = InMemoryCheckpointStore::new();
    let mut key = TransactionKey::from_envelope(&sample_envelope());
    key.transaction_id = " ".to_string();

    let error = store
        .apply_decision(&key)
        .await
        .expect_err("empty transaction id rejected");
    let message = error.to_string();
    assert!(message.contains("missing required field transaction_id"));

    let error = store
        .record_applied(key)
        .await
        .expect_err("empty transaction id not recorded");
    let message = error.to_string();
    assert!(message.contains("missing required field transaction_id"));
}

#[tokio::test]
async fn dedup_store_rejects_invalid_commit_lsn_identity() {
    let store = InMemoryCheckpointStore::new();
    let mut key = TransactionKey::from_envelope(&sample_envelope());
    key.commit_lsn = "0/0".to_string();

    let error = store
        .apply_decision(&key)
        .await
        .expect_err("zero commit lsn rejected");
    assert!(error.to_string().contains("commit_lsn"));
    assert!(error.to_string().contains("LSN must be greater than zero"));

    let error = store
        .record_applied(key)
        .await
        .expect_err("zero commit lsn not recorded");
    assert!(error.to_string().contains("commit_lsn"));
    assert!(error.to_string().contains("LSN must be greater than zero"));
}

#[tokio::test]
async fn checkpoint_store_merges_lsn_watermarks_with_ack_order() {
    let store = InMemoryCheckpointStore::new();
    let flow = FlowKey::new("source-a", "sales");
    store
        .save_checkpoint(Checkpoint {
            source_id: flow.source_id.clone(),
            dataset_id: flow.dataset_id.clone(),
            last_seen_lsn: "0/16B8000".to_string(),
            last_durable_lsn: "0/16B7800".to_string(),
            last_applied_lsn: "0/16B7000".to_string(),
        })
        .await
        .expect("seed checkpoint");

    store
        .save_checkpoint(Checkpoint {
            source_id: flow.source_id.clone(),
            dataset_id: flow.dataset_id.clone(),
            last_seen_lsn: "0/16B9000".to_string(),
            last_durable_lsn: "0/16B8200".to_string(),
            last_applied_lsn: "0/16B7900".to_string(),
        })
        .await
        .expect("merge checkpoint");

    let checkpoint = store
        .load_checkpoint(&flow)
        .await
        .expect("load checkpoint")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B9000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B8200");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B7900");
}

#[tokio::test]
async fn checkpoint_store_ignores_empty_incoming_lsn_watermarks() {
    let store = InMemoryCheckpointStore::new();
    let flow = FlowKey::new("source-a", "sales");
    store
        .save_checkpoint(Checkpoint {
            source_id: flow.source_id.clone(),
            dataset_id: flow.dataset_id.clone(),
            last_seen_lsn: "0/16B8000".to_string(),
            last_durable_lsn: "0/16B7800".to_string(),
            last_applied_lsn: "0/16B7000".to_string(),
        })
        .await
        .expect("seed checkpoint");

    store
        .save_checkpoint(Checkpoint {
            source_id: flow.source_id.clone(),
            dataset_id: flow.dataset_id.clone(),
            last_seen_lsn: String::new(),
            last_durable_lsn: String::new(),
            last_applied_lsn: String::new(),
        })
        .await
        .expect("merge checkpoint");

    let checkpoint = store
        .load_checkpoint(&flow)
        .await
        .expect("load checkpoint")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B7800");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B7000");
}

proptest! {
    #[test]
    fn in_memory_dedup_property_records_each_transaction_once(
        transactions in prop::collection::vec(transaction_key_strategy(), 1..64),
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");

        runtime.block_on(async move {
            let store = InMemoryCheckpointStore::new();
            let mut seen = StdHashSet::new();

            for transaction in transactions {
                let expected = if seen.contains(&transaction) {
                    ApplyDecision::SkipDuplicate
                } else {
                    ApplyDecision::Apply
                };

                prop_assert_eq!(store.apply_decision(&transaction).await.unwrap(), expected);
                store.record_applied(transaction.clone()).await.unwrap();
                prop_assert_eq!(
                    store.apply_decision(&transaction).await.unwrap(),
                    ApplyDecision::SkipDuplicate
                );
                seen.insert(transaction);
            }

            Ok(())
        })?;
    }
}
