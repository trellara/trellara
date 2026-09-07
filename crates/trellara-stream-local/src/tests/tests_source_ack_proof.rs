use super::*;
use std::io::Write;

use crate::frame::FRAME_MAGIC;

#[tokio::test]
async fn local_source_ack_proof_verifies_every_local_publish_ack() {
    let root = temp_root("source-ack-proof");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let messages = vec![message(topic, "tx-1", "one"), message(topic, "tx-2", "two")];
    let mut publish_acks = Vec::new();
    for message in messages.iter().cloned() {
        publish_acks.push(publisher.publish(message).await.expect("publish"));
    }

    let proof = local_source_ack_durability_proof(&root, &messages, &publish_acks, 2)
        .expect("source ACK durability proof");

    assert_eq!(proof.contract, LOCAL_SOURCE_ACK_DURABILITY_CONTRACT);
    assert_eq!(proof.durability, "fsync");
    assert!(proof.crash_safe_ack);
    assert_eq!(proof.expected_publish_messages, 2);
    assert_eq!(proof.durable_publish_acks, 2);
    assert!(proof.all_publish_acks_proven);
    assert_eq!(proof.ack_proofs.len(), 2);
    assert_eq!(proof.ack_proofs[0].key, "tx-1");
    assert_eq!(proof.ack_proofs[1].key, "tx-2");
    assert!(proof.ack_proofs.iter().all(|ack| ack.indexed));
    assert!(proof.ack_proofs.iter().all(|ack| ack.replayable));
    assert!(proof.ack_proofs.iter().all(|ack| ack.torn_tail_bytes == 0));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn buffered_source_ack_proof_is_visible_but_not_crash_safe() {
    let root = temp_root("source-ack-proof-buffered");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(
        LocalPublisherConfig::new(&root).with_durability(LocalDurability::Buffered),
    )
    .expect("publisher");
    let messages = vec![message(topic, "tx-1", "one")];
    let ack = publisher
        .publish(messages[0].clone())
        .await
        .expect("publish");

    let proof = local_source_ack_durability_proof_with_durability(
        &root,
        &messages,
        &[ack],
        1,
        LocalDurability::Buffered,
    )
    .expect("source ACK durability proof");

    assert_eq!(proof.contract, LOCAL_SOURCE_ACK_DURABILITY_CONTRACT);
    assert_eq!(proof.durability, "buffered");
    assert!(!proof.crash_safe_ack);
    assert!(proof.all_publish_acks_proven);
    assert_eq!(proof.ack_proofs[0].durability, "buffered");
    assert!(!proof.ack_proofs[0].crash_safe_ack);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_source_ack_proof_rejects_missing_or_wrong_ack_set() {
    let root = temp_root("source-ack-proof-rejects-ack-set");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let messages = vec![message(topic, "tx-1", "one"), message(topic, "tx-2", "two")];
    let first_ack = publisher
        .publish(messages[0].clone())
        .await
        .expect("publish first");

    assert!(matches!(
        local_source_ack_durability_proof(&root, &messages, &[first_ack], 2),
        Err(LocalStreamError::InvalidSourceAckDurabilityProof { reason })
            if reason.contains("expected 2 durable publish ACKs")
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_source_ack_proof_rejects_ack_topic_or_replayed_key_mismatch() {
    let root = temp_root("source-ack-proof-rejects-mismatch");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let messages = vec![message(topic, "tx-1", "one")];
    let ack = publisher
        .publish(messages[0].clone())
        .await
        .expect("publish");

    let mut wrong_topic = ack.clone();
    wrong_topic.topic = "trellara.source.dataset.other".to_string();
    assert!(matches!(
        local_source_ack_durability_proof(&root, &messages, &[wrong_topic], 1),
        Err(LocalStreamError::InvalidSourceAckDurabilityProof { reason })
            if reason.contains("does not match message topic")
    ));

    let wrong_key_message = vec![message(topic, "tx-other", "one")];
    assert!(matches!(
        local_source_ack_durability_proof(&root, &wrong_key_message, &[ack], 1),
        Err(LocalStreamError::InvalidSourceAckDurabilityProof { reason })
            if reason.contains("replays key")
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_source_ack_proof_rejects_torn_tail_before_source_ack() {
    let root = temp_root("source-ack-proof-rejects-torn-tail");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let messages = vec![message(topic, "tx-1", "one")];
    let ack = publisher
        .publish(messages[0].clone())
        .await
        .expect("publish");
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(topic_path(&root, topic))
            .expect("open topic");
        file.write_all(FRAME_MAGIC).expect("write partial magic");
        file.sync_all().expect("sync torn tail");
    }

    assert!(matches!(
        local_source_ack_durability_proof(&root, &messages, &[ack], 1),
        Err(LocalStreamError::InvalidPublishAckProof { topic: rejected, offset: 0, reason })
            if rejected == topic && reason.contains("not fully replayable")
    ));

    fs::remove_dir_all(root).expect("cleanup");
}
