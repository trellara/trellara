use super::*;
use trellara_protocol::{
    ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope,
};

#[test]
fn source_ack_boundary_rejects_missing_publish_acks() {
    let envelope = envelope("0/16B6C50");
    let messages = messages(&envelope);

    let error = SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(7)], 2, "0/16B6C50")
        .expect_err("missing durable publish ack rejected");

    assert!(matches!(
        error,
        RelayError::SourceAckBeforeDurablePublish {
            expected_publish_messages: 2,
            durable_publish_acks: 1,
        }
    ));
}

#[test]
fn source_ack_boundary_rejects_lsn_behind_commit() {
    let envelope = envelope("0/16B6C50");
    let messages = messages(&envelope);

    let error = SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(7)], 1, "0/16B6C00")
        .expect_err("stale source ack lsn rejected");

    assert!(matches!(
        error,
        RelayError::SourceAckLsnBehindCommit {
            commit_lsn,
            source_ack_lsn,
        } if commit_lsn == "0/16B6C50" && source_ack_lsn == "0/16B6C00"
    ));
}

#[test]
fn source_ack_boundary_canonicalizes_commit_and_ack_lsn() {
    let envelope = envelope("00000000/016B6C50");
    let messages = messages(&envelope);

    let proof =
        SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(7)], 1, "00000000/016B6C50")
            .expect("source ack proof");

    assert_eq!(proof.commit_lsn, "0/16B6C50");
    assert_eq!(proof.source_ack_lsn, "0/16B6C50");
}

#[test]
fn source_ack_boundary_rejects_wrong_publish_ack_destination() {
    let envelope = envelope("0/16B6C50");
    let messages = messages(&envelope);
    let wrong_topic_ack = PublishAck {
        topic: "trellara.other.sales.strict".to_string(),
        partition: 0,
        offset: 7,
    };

    let error =
        SourceAckBoundaryProof::recorded(&envelope, &messages, &[wrong_topic_ack], 1, "0/16B6C50")
            .expect_err("wrong stream destination rejected");

    assert!(matches!(
        error,
        RelayError::SourceAckPublishDestinationMismatch
    ));
}

#[test]
fn source_ack_boundary_rejects_invalid_publish_ack_offset() {
    let envelope = envelope("0/16B6C50");
    let messages = messages(&envelope);

    let error = SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(-1)], 1, "0/16B6C50")
        .expect_err("invalid publish offset rejected");

    assert!(matches!(
        error,
        RelayError::PublishAckOffsetInvalid { offset: -1 }
    ));
}

#[test]
fn source_ack_boundary_rejects_planned_message_without_partition() {
    let envelope = envelope("0/16B6C50");
    let mut messages = messages(&envelope);
    messages[0].partition = None;

    let error = SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(7)], 1, "0/16B6C50")
        .expect_err("missing planned partition rejected");

    assert!(matches!(
        error,
        RelayError::SourceAckPublishDestinationMissingPartition
    ));
}

#[test]
fn source_ack_boundary_records_expected_and_durable_destinations() {
    let envelope = envelope("0/16B6C50");
    let messages = messages(&envelope);

    let proof = SourceAckBoundaryProof::recorded(&envelope, &messages, &[ack(7)], 1, "0/16B6C50")
        .expect("source ack proof");

    assert_eq!(
        proof.expected_publish_destinations,
        proof.durable_publish_destinations
    );
    assert_eq!(proof.expected_publish_destination_count, 1);
    assert_eq!(proof.durable_publish_destination_count, 1);
    assert!(proof.publish_destinations_match);
    assert_eq!(
        proof.expected_publish_destinations[0].topic,
        "trellara.source.sales.strict"
    );
    assert_eq!(proof.expected_publish_destinations[0].partition, 0);
}

fn ack(offset: i64) -> PublishAck {
    PublishAck {
        topic: "trellara.source.sales.strict".to_string(),
        partition: 0,
        offset,
    }
}

fn messages(envelope: &TransactionEnvelope) -> Vec<StreamMessage> {
    vec![StreamMessage::strict_transaction(envelope).expect("strict stream message")]
}

fn envelope(commit_lsn: &str) -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ack-proof".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-ack-proof".to_string(),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            idempotency_key: "source:0/16B6C50:tx-ack-proof:1".to_string(),
            relation: Some(RelationId::new(42, "public", "sales")),
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id", 25, "store-1", true,
            )])),
        }],
    })
}
