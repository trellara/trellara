use super::*;
use trellara_protocol::{StrictEnvelope, TransactionEnvelope};
use trellara_relay::{SourceAckBoundaryProof, SOURCE_ACK_BOUNDARY_CONTRACT};
use trellara_stream::{PublishAck, StreamMessage};

fn step(publish_acks: Vec<PublishAck>) -> trellara_relay::RelayStep {
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1,
        changes: Vec::new(),
    });
    trellara_relay::RelayStep {
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

fn destination() -> trellara_relay::PublishDestination {
    trellara_relay::PublishDestination {
        topic: "topic".to_string(),
        partition: 0,
    }
}

fn ack(offset: i64) -> PublishAck {
    PublishAck {
        topic: "topic".to_string(),
        partition: 0,
        offset,
    }
}

#[test]
fn relay_summary_record_step_accumulates_checked_counts() {
    let mut summary = RelaySummary::default();

    summary.record_step(step(vec![ack(7)])).expect("relay step");

    assert_eq!(summary.published_transactions, 1);
    assert_eq!(summary.published_messages, 1);
    assert_eq!(summary.last_commit_lsn, Some("0/16B6C50".to_string()));
    assert_eq!(summary.last_topic, Some("topic".to_string()));
    assert_eq!(summary.last_partition, Some(0));
    assert_eq!(summary.last_offset, Some(7));
    assert_eq!(summary.latest_publish_messages.len(), 1);
    assert_eq!(summary.latest_publish_acks.len(), 1);
    assert!(summary.source_ack_after_durable_publish);
    assert!(summary.source_ack_publish_destinations_match);
    assert_eq!(summary.source_ack_publish_destination_count, 1);
    assert_eq!(summary.source_ack_lsn, Some("0/16B6C50".to_string()));
    assert_eq!(
        summary.source_ack_contract.as_deref(),
        Some(SOURCE_ACK_BOUNDARY_CONTRACT)
    );
}

#[test]
fn relay_summary_requires_all_expected_publish_acks_before_source_ack() {
    let mut summary = RelaySummary::default();
    let mut step = step(vec![ack(7)]);
    step.source_ack_boundary.expected_publish_messages = 2;
    step.source_ack_boundary.all_publish_acks_durable = false;

    summary.record_step(step).expect("relay step");

    assert!(!summary.source_ack_after_durable_publish);
}

#[test]
fn relay_summary_surfaces_publish_destination_mismatch() {
    let mut summary = RelaySummary::default();
    let mut step = step(vec![ack(7)]);
    step.source_ack_boundary.publish_destinations_match = false;

    summary.record_step(step).expect("relay step");

    assert!(!summary.source_ack_publish_destinations_match);
    assert!(!summary.source_ack_after_durable_publish);
    assert_eq!(summary.source_ack_publish_destination_count, 1);
}

#[test]
fn relay_summary_record_step_rejects_counter_overflow() {
    let mut summary = RelaySummary {
        published_messages: u64::MAX,
        ..RelaySummary::default()
    };

    let error = summary
        .record_step(step(vec![ack(7)]))
        .expect_err("overflow");

    assert!(matches!(
        error,
        CliError::RuntimeStatOverflow {
            field: "published_messages"
        }
    ));
}
