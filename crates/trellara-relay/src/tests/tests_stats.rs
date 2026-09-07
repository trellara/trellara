use super::*;
use trellara_protocol::{StrictEnvelope, TransactionEnvelope};
use trellara_stream::StreamMessage;

#[path = "tests_stats/destinations.rs"]
mod destinations;
#[path = "tests_stats/proof_fields.rs"]
mod proof_fields;

pub(super) fn relay_step(publish_acks: Vec<PublishAck>) -> RelayStep {
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "db".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1,
        changes: Vec::new(),
    });
    RelayStep {
        published_messages: vec![
            StreamMessage::strict_transaction(&envelope).expect("stream message")
        ],
        envelope,
        publish_acks,
        source_ack_lsn: "0/16B6C50".to_string(),
        source_ack_boundary: SourceAckBoundaryProof {
            contract: SOURCE_ACK_BOUNDARY_CONTRACT,
            source_id: "source".to_string(),
            dataset_id: "sales".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            source_ack_lsn: "0/16B6C50".to_string(),
            expected_publish_messages: 1,
            durable_publish_acks: 1,
            expected_publish_destination_count: 1,
            durable_publish_destination_count: 1,
            expected_publish_destinations: vec![destination()],
            durable_publish_destinations: vec![destination()],
            publish_destinations_match: true,
            all_publish_acks_durable: true,
            last_publish_ack: Some(ack(7)),
            durable_lsn_covers_commit: true,
            checkpoint_recorded_before_source_ack: true,
        },
    }
}

fn destination() -> PublishDestination {
    PublishDestination {
        topic: "topic".to_string(),
        partition: 0,
    }
}

pub(super) fn ack(offset: i64) -> PublishAck {
    PublishAck {
        topic: "topic".to_string(),
        partition: 0,
        offset,
    }
}

#[test]
fn relay_run_stats_record_step_accumulates_counts() {
    let mut stats = RelayRunStats::default();

    stats
        .record_step(relay_step(vec![ack(7)]))
        .expect("record step");

    assert_eq!(stats.published_transactions, 1);
    assert_eq!(stats.published_messages, 1);
    assert_eq!(stats.last_commit_lsn, Some("0/16B6C50".to_string()));
    assert_eq!(stats.last_ack.expect("ack").offset, 7);
    assert_eq!(stats.last_published_messages.len(), 1);
    assert_eq!(stats.last_publish_acks.len(), 1);
    let proof = stats.last_source_ack_boundary.expect("source ack proof");
    assert_eq!(proof.source_ack_lsn, "0/16B6C50");
    assert!(proof.all_publish_acks_durable);
    assert!(proof.checkpoint_recorded_before_source_ack);
}

#[test]
fn relay_run_stats_record_step_rejects_counter_overflow() {
    let mut stats = RelayRunStats {
        published_messages: u64::MAX,
        ..RelayRunStats::default()
    };

    let error = stats
        .record_step(relay_step(vec![ack(7)]))
        .expect_err("overflow");

    assert!(matches!(
        error,
        RelayError::StatOverflow {
            field: "published_messages"
        }
    ));
}

#[test]
fn relay_run_stats_rejects_source_ack_identity_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.dataset_id = "warehouse".to_string();

    let error = stats
        .record_step(step)
        .expect_err("source ack proof identity mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "dataset_id",
            step,
            proof,
        } if step == "\"sales\"" && proof == "\"warehouse\""
    ));
}
