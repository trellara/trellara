use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use trellara_stream::{StreamHeader, StreamMessage};

use super::*;
use crate::publisher_outcome::{classify_publish_error, validate_kafka_message_for_publish};

fn message() -> StreamMessage {
    StreamMessage {
        topic: "trellara.source.dataset.strict".to_string(),
        key: "tx-1".to_string(),
        payload: Vec::new().into(),
        headers: vec![StreamHeader::new("trellara.transaction_id", "tx-1")],
        partition: Some(0),
        position: None,
    }
}

#[test]
fn kafka_publish_validation_rejects_empty_key() {
    let mut message = message();
    message.key.clear();

    assert!(validate_kafka_message_for_publish(&message).is_err());
}

#[test]
fn kafka_publish_validation_rejects_duplicate_header_keys() {
    let mut message = message();
    message
        .headers
        .push(StreamHeader::new("trellara.transaction_id", "tx-1"));

    assert!(validate_kafka_message_for_publish(&message).is_err());
}

#[test]
fn request_timeout_requires_identity_reconciliation() {
    let message = message();
    let identity = KafkaPublishIdentity::from_message(&message);

    let failure = classify_publish_error(
        KafkaError::MessageProduction(RDKafkaErrorCode::RequestTimedOut),
        identity.clone(),
    );

    assert!(matches!(
        failure,
        KafkaPublishFailure::Ambiguous {
            identity: actual,
            ..
        } if actual == identity
    ));
}

#[test]
fn queue_backpressure_is_definitively_not_published() {
    let failure = classify_publish_error(
        KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull),
        KafkaPublishIdentity::from_message(&message()),
    );

    assert!(matches!(failure, KafkaPublishFailure::Rejected { .. }));
}

#[test]
fn reconciliation_identity_matches_consumed_copy() {
    let outgoing = message();
    let identity = KafkaPublishIdentity::from_message(&outgoing);
    let mut observed = outgoing.clone();
    observed.partition = None;
    observed.position = Some(trellara_stream::StreamPosition {
        topic: observed.topic.clone(),
        partition: 0,
        offset: 42,
    });

    assert!(identity.matches(&observed));
    observed.payload = vec![1].into();
    assert!(!identity.matches(&observed));
}

#[test]
fn unpinned_reconciliation_identity_accepts_broker_selected_partition() {
    let mut outgoing = message();
    outgoing.partition = None;
    let identity = KafkaPublishIdentity::from_message(&outgoing);
    outgoing.position = Some(trellara_stream::StreamPosition {
        topic: outgoing.topic.clone(),
        partition: 9,
        offset: 42,
    });

    assert!(identity.matches(&outgoing));
}
