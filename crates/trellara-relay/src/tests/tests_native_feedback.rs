use super::*;
use trellara_pg_extension::{
    native_committed_transaction_frame, native_handoff_drain_batch, native_handoff_queue_config,
};
use trellara_protocol::{
    ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope,
};
use trellara_stream::StreamMessage;

#[test]
fn native_feedback_proof_maps_relay_source_ack_boundary() {
    let step = relay_step();

    let proof = native_feedback_proof_from_relay_step(&step, vec![step.envelope.checksum])
        .expect("native feedback proof");

    assert_eq!(proof.durable_publish_lsn, "0/16B6C50");
    assert_eq!(
        proof.published_frame_checksums,
        vec![step.envelope.checksum]
    );
    assert_eq!(
        proof.expected_publish_destinations,
        proof.durable_publish_destinations
    );
    assert_eq!(
        proof.expected_publish_destinations[0].topic,
        "trellara.source.sales.strict"
    );
    assert_eq!(proof.expected_publish_destinations[0].partition, 0);
}

#[test]
fn native_feedback_proof_rejects_undurable_relay_boundary() {
    let mut step = relay_step();
    step.source_ack_boundary
        .checkpoint_recorded_before_source_ack = false;

    assert!(matches!(
        native_feedback_proof_from_relay_step(&step, vec![step.envelope.checksum]),
        Err(RelayError::NativeFeedbackProofNotDurable {
            field: "checkpoint_recorded_before_source_ack"
        })
    ));
}

#[test]
fn native_feedback_decision_covers_native_worker_handoff_simulation() {
    let step = relay_step();
    let batch = native_batch(step.envelope.checksum, "0/16B6C50");

    let decision =
        native_feedback_decision_from_relay_step(&batch, &step).expect("native decision");

    assert_eq!(decision.source_feedback_lsn, "0/16B6C50");
    assert_eq!(decision.frames_covered, 1);
}

#[test]
fn native_feedback_decision_rejects_relay_step_behind_native_batch() {
    let mut step = relay_step();
    step.source_ack_boundary.source_ack_lsn = "0/16B6C50".to_string();
    let batch = native_batch(step.envelope.checksum, "0/16B6C60");

    assert!(matches!(
        native_feedback_decision_from_relay_step(&batch, &step),
        Err(RelayError::NativeFeedbackRejected(
            trellara_pg_extension::NativeSourceFeedbackError::DurableLsnBehindBatch { .. }
        ))
    ));
}

fn relay_step() -> RelayStep {
    let envelope = envelope();
    let messages = vec![StreamMessage::strict_transaction(&envelope).expect("stream message")];
    let publish_acks = vec![ack()];
    let source_ack_boundary = SourceAckBoundaryProof::recorded(
        &envelope,
        &messages,
        &publish_acks,
        messages.len(),
        "0/16B6C50",
    )
    .expect("source ack proof");

    RelayStep {
        envelope,
        published_messages: messages,
        publish_acks,
        source_ack_lsn: "0/16B6C50".to_string(),
        source_ack_boundary,
    }
}

fn ack() -> PublishAck {
    PublishAck {
        topic: "trellara.source.sales.strict".to_string(),
        partition: 0,
        offset: 7,
    }
}

fn native_batch(checksum: u64, commit_lsn: &str) -> trellara_pg_extension::NativeHandoffDrainBatch {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");
    let frame = native_committed_transaction_frame(
        "source",
        "sales",
        "tx-native-feedback",
        commit_lsn,
        1024,
        checksum,
    )
    .expect("native handoff frame");
    native_handoff_drain_batch(&config, vec![frame]).expect("native handoff batch")
}

fn envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-native-feedback".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-native-feedback".to_string(),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            idempotency_key: "source:0/16B6C50:tx-native-feedback:1".to_string(),
            relation: Some(RelationId::new(42, "public", "sales")),
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id", 25, "store-1", true,
            )])),
        }],
    });
    envelope.finalize_checksum();
    envelope
}
