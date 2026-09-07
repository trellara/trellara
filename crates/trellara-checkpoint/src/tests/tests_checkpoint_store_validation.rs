use super::*;
use trellara_protocol::Checkpoint;

fn checkpoint(last_seen_lsn: &str, last_durable_lsn: &str, last_applied_lsn: &str) -> Checkpoint {
    Checkpoint {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: last_seen_lsn.to_string(),
        last_durable_lsn: last_durable_lsn.to_string(),
        last_applied_lsn: last_applied_lsn.to_string(),
    }
}

#[tokio::test]
async fn checkpoint_store_rejects_empty_flow_identity() {
    let store = InMemoryCheckpointStore::new();
    let mut checkpoint = checkpoint("0/16B8000", "0/16B7800", "0/16B7000");
    checkpoint.source_id = " ".to_string();

    let error = store
        .save_checkpoint(checkpoint)
        .await
        .expect_err("empty source rejected");

    assert!(error.to_string().contains("source_id must not be empty"));
}

#[tokio::test]
async fn checkpoint_store_rejects_flow_identity_with_surrounding_whitespace() {
    for (source_id, dataset_id, field) in [
        (" source-a ", "sales", "source_id"),
        ("source-a", " sales ", "dataset_id"),
    ] {
        let store = InMemoryCheckpointStore::new();
        let mut checkpoint = checkpoint("0/16B8000", "0/16B7800", "0/16B7000");
        checkpoint.source_id = source_id.to_string();
        checkpoint.dataset_id = dataset_id.to_string();

        let error = store
            .save_checkpoint(checkpoint)
            .await
            .expect_err("spaced checkpoint identity rejected");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}

#[tokio::test]
async fn checkpoint_store_rejects_load_with_non_canonical_flow_key() {
    for (source_id, dataset_id, field) in [
        (" source-a ", "sales", "source_id"),
        ("source-a", " sales ", "dataset_id"),
    ] {
        let error = InMemoryCheckpointStore::new()
            .load_checkpoint(&FlowKey::new(source_id, dataset_id))
            .await
            .expect_err("spaced load flow rejected");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}

#[tokio::test]
async fn checkpoint_store_rejects_invalid_lsn_evidence() {
    let store = InMemoryCheckpointStore::new();

    let error = store
        .save_checkpoint(checkpoint("bad-lsn", "0/16B7800", "0/16B7000"))
        .await
        .expect_err("invalid seen lsn rejected");

    assert!(error.to_string().contains("last_seen_lsn"));
    assert!(error.to_string().contains("non-zero PostgreSQL LSN"));

    let error = store
        .save_checkpoint(checkpoint("0/16B8000", "0/0", "0/16B7000"))
        .await
        .expect_err("zero durable lsn rejected");

    assert!(error.to_string().contains("last_durable_lsn"));
    assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
}

#[tokio::test]
async fn checkpoint_store_rejects_durable_lsn_ahead_of_seen_lsn() {
    let store = InMemoryCheckpointStore::new();

    let error = store
        .save_checkpoint(checkpoint("0/16B7000", "0/16B8000", "0/16B7000"))
        .await
        .expect_err("durable ack cannot outrun seen wal");

    assert!(error.to_string().contains("last_durable_lsn"));
    assert!(error
        .to_string()
        .contains("must not be ahead of last_seen_lsn"));
}

#[tokio::test]
async fn checkpoint_store_rejects_applied_lsn_ahead_of_durable_lsn() {
    let store = InMemoryCheckpointStore::new();

    let error = store
        .save_checkpoint(checkpoint("0/16B9000", "0/16B7000", "0/16B8000"))
        .await
        .expect_err("target apply cannot outrun durable boundary");

    assert!(error.to_string().contains("last_applied_lsn"));
    assert!(error
        .to_string()
        .contains("must not be ahead of last_durable_lsn"));
}

#[tokio::test]
async fn checkpoint_store_still_accepts_empty_partial_lsn_updates() {
    let store = InMemoryCheckpointStore::new();
    let flow = FlowKey::new("source-a", "sales");

    store
        .save_checkpoint(checkpoint("0/16B8000", "0/16B7800", "0/16B7000"))
        .await
        .expect("seed checkpoint");
    store
        .save_checkpoint(checkpoint("", "", ""))
        .await
        .expect("empty partial update ignored by merge");

    let checkpoint = store
        .load_checkpoint(&flow)
        .await
        .expect("load checkpoint")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B7800");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B7000");
}

#[tokio::test]
async fn checkpoint_store_refuses_non_empty_rewind_evidence() {
    let store = InMemoryCheckpointStore::new();
    let flow = FlowKey::new("source-a", "sales");
    store
        .save_checkpoint(checkpoint("0/16B9000", "0/16B8800", "0/16B8600"))
        .await
        .expect("seed checkpoint");

    let error = store
        .save_checkpoint(checkpoint("0/16BA000", "0/16B8700", "0/16B8600"))
        .await
        .expect_err("durable rewind refused");

    let message = error.to_string();
    assert!(message.contains("checkpoint rewind refused"));
    assert!(message.contains("last_durable_lsn"));
    assert!(message.contains("0/16B8700"));
    assert!(message.contains("0/16B8800"));

    let checkpoint = store
        .load_checkpoint(&flow)
        .await
        .expect("load checkpoint")
        .expect("checkpoint remains");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B9000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B8800");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B8600");
}
