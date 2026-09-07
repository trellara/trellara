use super::*;
use trellara_protocol::{
    ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope,
};
use trellara_stream::{PublishAck, StreamMessage};

#[test]
fn relay_publish_proof_records_ordered_durable_ack_boundary() {
    let envelope = envelope();
    let messages = vec![StreamMessage::strict_transaction(&envelope).expect("strict message")];
    let publish_acks = vec![ack(7)];

    let proof =
        RelayPublishProof::recorded(&messages, &publish_acks, 1).expect("relay publish proof");

    assert_eq!(proof.contract, RELAY_PUBLISH_PROOF_CONTRACT);
    assert_eq!(proof.expected_publish_messages, 1);
    assert_eq!(proof.durable_publish_acks, 1);
    assert_eq!(
        proof.expected_publish_destinations,
        proof.durable_publish_destinations
    );
    assert_eq!(proof.ordered_ack_offsets, vec![7]);
    assert_eq!(proof.last_publish_ack, Some(ack(7)));
    assert!(proof.all_publish_acks_durable);
}

#[test]
fn relay_publish_proof_rejects_missing_wrong_or_invalid_acks() {
    let envelope = envelope();
    let messages = vec![StreamMessage::strict_transaction(&envelope).expect("strict message")];

    assert!(matches!(
        RelayPublishProof::recorded(&messages, &[], 1),
        Err(RelayError::SourceAckBeforeDurablePublish {
            expected_publish_messages: 1,
            durable_publish_acks: 0,
        })
    ));

    assert!(matches!(
        RelayPublishProof::recorded(
            &messages,
            &[PublishAck {
                topic: "trellara.other.sales.strict".to_string(),
                partition: 0,
                offset: 7,
            }],
            1,
        ),
        Err(RelayError::SourceAckPublishDestinationMismatch)
    ));

    assert!(matches!(
        RelayPublishProof::recorded(&messages, &[ack(-1)], 1),
        Err(RelayError::PublishAckOffsetInvalid { offset: -1 })
    ));
}

#[test]
fn relay_publish_proof_rejects_non_advancing_offsets_per_destination() {
    let envelope = envelope();
    let message = StreamMessage::strict_transaction(&envelope).expect("strict message");
    let messages = vec![message.clone(), message];

    assert!(matches!(
        RelayPublishProof::recorded(&messages, &[ack(7), ack(7)], 2),
        Err(RelayError::PublishAckOffsetNotAdvancing {
            topic,
            partition: 0,
            previous_offset: 7,
            offset: 7,
        }) if topic == "trellara.source.sales.strict"
    ));

    assert!(matches!(
        RelayPublishProof::recorded(&messages, &[ack(8), ack(7)], 2),
        Err(RelayError::PublishAckOffsetNotAdvancing {
            topic,
            partition: 0,
            previous_offset: 8,
            offset: 7,
        }) if topic == "trellara.source.sales.strict"
    ));
}

fn ack(offset: i64) -> PublishAck {
    PublishAck {
        topic: "trellara.source.sales.strict".to_string(),
        partition: 0,
        offset,
    }
}

fn envelope() -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-publish-proof".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-publish-proof".to_string(),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            idempotency_key: "source:0/16B6C50:tx-publish-proof:1".to_string(),
            relation: Some(RelationId::new(42, "public", "sales")),
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id", 25, "store-1", true,
            )])),
        }],
    })
}
