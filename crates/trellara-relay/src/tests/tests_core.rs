use super::*;
use trellara_checkpoint::InMemoryCheckpointStore;

#[tokio::test]
async fn run_once_publishes_transaction_and_records_durable_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![envelope("tx-1", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(source, publisher.clone(), checkpoint_store.clone());

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published transaction");

    assert_eq!(step.envelope.transaction_id, "tx-1");
    assert_eq!(step.publish_acks.len(), 1);
    assert_eq!(
        step.source_ack_boundary.contract,
        SOURCE_ACK_BOUNDARY_CONTRACT
    );
    assert_eq!(step.source_ack_boundary.commit_lsn, "0/16B6C50");
    assert_eq!(step.source_ack_boundary.source_ack_lsn, "0/16B6C50");
    assert_eq!(step.source_ack_boundary.expected_publish_messages, 1);
    assert_eq!(step.source_ack_boundary.durable_publish_acks, 1);
    assert!(step.source_ack_boundary.all_publish_acks_durable);
    assert!(step.source_ack_boundary.durable_lsn_covers_commit);
    assert!(
        step.source_ack_boundary
            .checkpoint_recorded_before_source_ack
    );
    assert_eq!(publisher.published_messages().len(), 1);

    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B6C50");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
    assert_eq!(checkpoint.last_applied_lsn, "");
    assert_eq!(
        *acked_lsns.lock().expect("acked lsn lock"),
        vec!["0/16B6C50".to_string()]
    );
}

#[tokio::test]
async fn strict_mode_publishes_ddl_transaction_with_release_gate_headers() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![ddl_envelope("tx-ddl", "0/16B6C50")]);
    let mut relay = Relay::new(source, publisher.clone(), checkpoint_store);

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published transaction");

    assert_eq!(step.envelope.ddl_events.len(), 1);
    let messages = publisher.published_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.ddl_event_count",
            "1"
        )));
    assert!(messages[0]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.ddl_release_gates",
            "post_ddl_dml_release"
        )));
    assert!(messages[0]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
        "trellara.ddl_propagation_decisions",
        "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
    )));
    assert!(messages[0]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.ddl_target_ack_required",
            "1"
        )));
    assert!(messages[0].headers.iter().any(|header| header.key
        == "trellara.ddl_propagation_policy_sha256"
        && header.value.len() == 64
        && header
            .value
            .chars()
            .all(|character| character.is_ascii_hexdigit())));
}

#[tokio::test]
async fn run_until_idle_publishes_all_available_transactions() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let mut relay = Relay::new(
        FakeSource::new(vec![
            envelope("tx-1", "0/16B6C50"),
            envelope("tx-2", "0/16B6D00"),
        ]),
        publisher.clone(),
        checkpoint_store,
    );

    let stats = relay.run_until_idle().await.expect("relay run");

    assert_eq!(stats.published_transactions, 2);
    assert_eq!(stats.published_messages, 2);
    assert_eq!(stats.last_commit_lsn, Some("0/16B6D00".to_string()));
    assert_eq!(stats.last_ack.expect("last ack").offset, 1);
    let proof = stats
        .last_source_ack_boundary
        .expect("last source ack boundary");
    assert_eq!(proof.commit_lsn, "0/16B6D00");
    assert_eq!(proof.source_ack_lsn, "0/16B6D00");
    assert_eq!(proof.expected_publish_messages, 1);
    assert!(proof.all_publish_acks_durable);
    assert!(proof.checkpoint_recorded_before_source_ack);
    assert_eq!(publisher.published_messages().len(), 2);
}

#[tokio::test]
async fn run_once_returns_none_when_source_is_idle() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let mut relay = Relay::new(
        FakeSource::new(Vec::new()),
        RecordingPublisher::succeeding(),
        checkpoint_store,
    );

    assert!(relay.run_once().await.expect("relay idle").is_none());
}
