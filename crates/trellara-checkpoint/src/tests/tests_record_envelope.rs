use super::*;
use trellara_protocol::Checkpoint;

#[tokio::test]
async fn recording_envelope_updates_checkpoint() {
    let store = InMemoryCheckpointStore::new();
    let envelope = sample_envelope();

    store.record_envelope_applied(&envelope).await.unwrap();
    let checkpoint = store
        .load_checkpoint(&FlowKey::from(&envelope))
        .await
        .unwrap()
        .expect("checkpoint");

    assert_checkpoint_watermark(&checkpoint, "0/16B6C50");
}

#[tokio::test]
async fn recording_envelope_canonicalizes_checkpoint_identity_lsn() {
    let store = InMemoryCheckpointStore::new();
    let mut envelope = sample_envelope();
    envelope.commit_lsn = "00000000/016B6C50".to_string();
    envelope.finalize_checksum();

    store.record_envelope_applied(&envelope).await.unwrap();
    let checkpoint = store
        .load_checkpoint(&FlowKey::from(&envelope))
        .await
        .unwrap()
        .expect("checkpoint");

    assert_checkpoint_watermark(&checkpoint, "0/16B6C50");
}

#[tokio::test]
async fn recording_envelope_rejects_invalid_transaction_identity() {
    let store = InMemoryCheckpointStore::new();
    let mut envelope = sample_envelope();
    envelope.transaction_id.clear();
    envelope.finalize_checksum();

    let error = store
        .record_envelope_applied(&envelope)
        .await
        .expect_err("missing transaction id rejected");

    assert!(error
        .to_string()
        .contains("missing required field transaction_id"));
}

fn assert_checkpoint_watermark(checkpoint: &Checkpoint, lsn: &str) {
    assert_eq!(checkpoint.last_seen_lsn, lsn);
    assert_eq!(checkpoint.last_durable_lsn, lsn);
    assert_eq!(checkpoint.last_applied_lsn, lsn);
}
