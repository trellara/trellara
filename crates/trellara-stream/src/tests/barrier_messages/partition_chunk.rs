use super::helpers::{assert_barrier_payload_mismatch, partition_plan};
use super::*;

#[test]
fn partition_chunk_message_uses_partition_topic_key_and_headers() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let chunk = plan.chunks.first().expect("chunk");

    let message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk message");

    assert_eq!(
        message.topic,
        format!("trellara.source_a.sales.partition.{}", chunk.partition_id)
    );
    assert_eq!(message.partition, Some(chunk.partition_id as i32));
    assert_eq!(
        message.key,
        format!(
            "source_a:retail:sales:tx-1:0/16B6C50:{}",
            chunk.partition_id
        )
    );
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.message_kind",
        "partition_chunk"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_decision",
        "partition_parallel_dml"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.dml_replay_after_ddl_barrier_required",
        "false"
    )));
    let decoded = PartitionChunk::decode(message.payload.as_ref()).expect("decode");
    assert_eq!(decoded.transaction_id, "tx-1");
    assert_eq!(decoded.partition_id, chunk.partition_id);
}

#[test]
fn partition_chunk_message_rejects_partition_id_overflow() {
    let envelope = sample_envelope();
    let partition_id = i32::MAX as u32 + 1;
    let chunk = PartitionChunk::new(
        envelope.transaction_id.clone(),
        partition_id,
        envelope.changes.clone(),
    );

    assert!(matches!(
        StreamMessage::partition_chunk(&envelope, &chunk),
        Err(StreamError::InvalidStreamPartition {
            partition_id: actual,
            max_supported,
        }) if actual == partition_id && max_supported == i32::MAX as u32
    ));
}

#[test]
fn strict_chunk_message_uses_strict_topic_with_chunk_headers() {
    let envelope = sample_envelope();
    let chunk = PartitionChunk::new(envelope.transaction_id.clone(), 7, envelope.changes.clone());

    let message = StreamMessage::strict_chunk(&envelope, &chunk).expect("chunk message");

    assert_eq!(message.topic, "trellara.source_a.sales.strict");
    assert_eq!(message.partition, Some(0));
    assert_eq!(message.key, "source_a:retail:sales:tx-1:0/16B6C50:7");
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.message_kind", "strict_chunk")));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.partition_id", "7")));
    let decoded = PartitionChunk::decode(message.payload.as_ref()).expect("decode");
    assert_eq!(decoded.partition_id, 7);
}

#[test]
fn chunk_message_rejects_envelope_transaction_mismatch() {
    let envelope = sample_envelope();
    let mut changes = envelope.changes.clone();
    for change in &mut changes {
        change.transaction_id = "tx-other".to_string();
    }
    let chunk = PartitionChunk::new("tx-other", 7, changes);

    assert_barrier_payload_mismatch(
        StreamMessage::partition_chunk(&envelope, &chunk),
        "transaction_id",
    );
    assert_barrier_payload_mismatch(
        StreamMessage::strict_chunk(&envelope, &chunk),
        "transaction_id",
    );
}

#[test]
fn chunk_message_rejects_stale_chunk_checksum() {
    let envelope = sample_envelope();
    let mut chunk =
        PartitionChunk::new(envelope.transaction_id.clone(), 7, envelope.changes.clone());
    chunk.checksum = chunk.checksum.wrapping_add(1);

    assert!(matches!(
        StreamMessage::partition_chunk(&envelope, &chunk),
        Err(StreamError::Protocol(
            ProtocolError::PartitionChecksumMismatch {
                partition_id: 7,
                ..
            }
        ))
    ));
}

#[test]
fn chunk_messages_reject_change_total_order_missing_from_envelope() {
    let envelope = sample_envelope();
    let mut changes = envelope.changes.clone();
    changes[0].total_order = 2;
    changes[0].table_order = 2;
    changes[0].partition_order = 2;
    let chunk = PartitionChunk::new(envelope.transaction_id.clone(), 7, changes);

    assert_barrier_payload_mismatch(
        StreamMessage::partition_chunk(&envelope, &chunk),
        "chunk.change.total_order",
    );
    assert_barrier_payload_mismatch(
        StreamMessage::strict_chunk(&envelope, &chunk),
        "chunk.change.total_order",
    );
}

#[test]
fn chunk_messages_reject_change_record_drift_from_envelope() {
    let envelope = sample_envelope();
    let mut changes = envelope.changes.clone();
    changes[0].idempotency_key = "source_a:0/16B6C50:tx-1:drifted".to_string();
    let chunk = PartitionChunk::new(envelope.transaction_id.clone(), 7, changes);

    assert_barrier_payload_mismatch(
        StreamMessage::partition_chunk(&envelope, &chunk),
        "chunk.change",
    );
    assert_barrier_payload_mismatch(
        StreamMessage::strict_chunk(&envelope, &chunk),
        "chunk.change",
    );
}

#[test]
fn chunk_messages_reject_duplicate_total_order_before_publish() {
    let envelope = sample_envelope();
    let mut chunk =
        PartitionChunk::new(envelope.transaction_id.clone(), 7, envelope.changes.clone());
    chunk.changes.push(envelope.changes[0].clone());
    chunk.finalize_checksum();

    assert!(matches!(
        StreamMessage::partition_chunk(&envelope, &chunk),
        Err(StreamError::Protocol(
            ProtocolError::DuplicateTransactionEventOrder { total_order: 1 }
        ))
    ));
    assert!(matches!(
        StreamMessage::strict_chunk(&envelope, &chunk),
        Err(StreamError::Protocol(
            ProtocolError::DuplicateTransactionEventOrder { total_order: 1 }
        ))
    ));
}

#[test]
fn chunk_messages_reject_invalid_chunk_identity_before_publish() {
    let envelope = sample_envelope();
    let mut chunk =
        PartitionChunk::new(envelope.transaction_id.clone(), 7, envelope.changes.clone());
    chunk.transaction_id = " tx-1".to_string();
    chunk.finalize_checksum();

    assert!(matches!(
        StreamMessage::partition_chunk(&envelope, &chunk),
        Err(StreamError::Protocol(
            ProtocolError::InvalidPartitionChunkField {
                field: "transaction_id",
                ..
            }
        ))
    ));
    assert!(matches!(
        StreamMessage::strict_chunk(&envelope, &chunk),
        Err(StreamError::Protocol(
            ProtocolError::InvalidPartitionChunkField {
                field: "transaction_id",
                ..
            }
        ))
    ));
}
