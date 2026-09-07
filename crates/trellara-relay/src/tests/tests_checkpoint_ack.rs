use super::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use trellara_checkpoint::{FlowKey, InMemoryCheckpointStore};

#[derive(Clone)]
struct LegacyCheckpointStore {
    checkpoint: Arc<Mutex<Option<Checkpoint>>>,
}

impl LegacyCheckpointStore {
    fn with_checkpoint(checkpoint: Checkpoint) -> Self {
        Self {
            checkpoint: Arc::new(Mutex::new(Some(checkpoint))),
        }
    }
}

#[async_trait]
impl CheckpointStore for LegacyCheckpointStore {
    async fn load_checkpoint(
        &self,
        _flow: &FlowKey,
    ) -> trellara_checkpoint::Result<Option<Checkpoint>> {
        Ok(self.checkpoint.lock().expect("checkpoint lock").clone())
    }

    async fn save_checkpoint(&self, checkpoint: Checkpoint) -> trellara_checkpoint::Result<()> {
        *self.checkpoint.lock().expect("checkpoint lock") = Some(checkpoint);
        Ok(())
    }
}

#[tokio::test]
async fn failed_publish_does_not_advance_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let source = FakeSource::new(vec![envelope("tx-1", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(
        source,
        RecordingPublisher::failing(),
        checkpoint_store.clone(),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn invalid_envelope_lsn_does_not_advance_checkpoint_or_source_ack() {
    for commit_lsn in ["not-an-lsn", "100000000/0", "0/100000000"] {
        let checkpoint_store = InMemoryCheckpointStore::new();
        let publisher = RecordingPublisher::succeeding();
        let source = FakeSource::new(vec![envelope("tx-invalid", commit_lsn)]);
        let acked_lsns = source.acked_lsns();
        let mut relay = Relay::new(source, publisher.clone(), checkpoint_store.clone());

        assert!(matches!(
            relay.run_once().await,
            Err(RelayError::Stream(StreamError::Protocol(
                ProtocolError::InvalidLsn { .. }
            )))
        ));
        assert!(publisher.published_messages().is_empty());
        assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
            .await
            .expect("checkpoint load")
            .is_none());
        assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
    }
}

#[tokio::test]
async fn invalid_existing_checkpoint_lsn_blocks_source_ack() {
    let checkpoint_store = LegacyCheckpointStore::with_checkpoint(Checkpoint {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: "also-not-an-lsn".to_string(),
        last_durable_lsn: "not-an-lsn".to_string(),
        last_applied_lsn: "0/16B6800".to_string(),
    });
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![envelope("tx-new", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(source, publisher.clone(), checkpoint_store.clone());

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Protocol(ProtocolError::InvalidLsn { .. }))
    ));
    assert_eq!(publisher.published_messages().len(), 1);
    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_durable_lsn, "not-an-lsn");
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn inconsistent_existing_checkpoint_watermarks_block_source_ack() {
    let checkpoint_store = LegacyCheckpointStore::with_checkpoint(Checkpoint {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: "0/16B7000".to_string(),
        last_durable_lsn: "0/16B8000".to_string(),
        last_applied_lsn: "0/16B7000".to_string(),
    });
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![envelope("tx-stale-corrupt-checkpoint", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(source, publisher.clone(), checkpoint_store.clone());

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::CheckpointWatermarkInconsistent {
            field: "last_durable_lsn",
            boundary_field: "last_seen_lsn",
            ..
        })
    ));
    assert_eq!(publisher.published_messages().len(), 1);
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn source_ack_failure_happens_after_durable_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let mut relay = Relay::new(
        FakeSource::with_ack_failure(vec![envelope("tx-1", "0/16B6C50")]),
        RecordingPublisher::succeeding(),
        checkpoint_store.clone(),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Capture(CaptureError::ReplicationProtocol(_)))
    ));

    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
}
