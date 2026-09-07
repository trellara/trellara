use super::*;

#[test]
fn source_feedback_advances_only_after_durable_frame_proof() {
    let batch = drain_batch();
    let proof = NativeSourceFeedbackProof {
        durable_publish_lsn: "0/16B6C60".to_string(),
        published_frame_checksums: vec![7, 8],
        expected_publish_destinations: destinations(),
        durable_publish_destinations: destinations(),
    };

    let decision = native_source_feedback_decision(&batch, proof).expect("feedback decision");

    assert_eq!(decision.source_feedback_lsn, "0/16B6C60");
    assert_eq!(decision.frames_covered, 2);
    assert_eq!(decision.contract, SOURCE_ACKNOWLEDGEMENT_CONTRACT);
}

#[test]
fn source_feedback_rejects_missing_or_mismatched_frame_proof() {
    let batch = drain_batch();

    assert_eq!(
        native_source_feedback_decision(
            &batch,
            NativeSourceFeedbackProof {
                durable_publish_lsn: "0/16B6C60".to_string(),
                published_frame_checksums: vec![7],
                expected_publish_destinations: destinations(),
                durable_publish_destinations: destinations(),
            },
        ),
        Err(NativeSourceFeedbackError::FrameProofCountMismatch {
            expected_frames: 2,
            published_frames: 1,
        })
    );
    assert_eq!(
        native_source_feedback_decision(
            &batch,
            NativeSourceFeedbackProof {
                durable_publish_lsn: "0/16B6C60".to_string(),
                published_frame_checksums: vec![7, 99],
                expected_publish_destinations: destinations(),
                durable_publish_destinations: destinations(),
            },
        ),
        Err(NativeSourceFeedbackError::FrameProofChecksumMismatch {
            frame_index: 1,
            expected_checksum: 8,
            published_checksum: 99,
        })
    );
}

#[test]
fn source_feedback_rejects_durable_lsn_behind_drained_batch() {
    let batch = drain_batch();

    assert_eq!(
        native_source_feedback_decision(
            &batch,
            NativeSourceFeedbackProof {
                durable_publish_lsn: "0/16B6C50".to_string(),
                published_frame_checksums: vec![7, 8],
                expected_publish_destinations: destinations(),
                durable_publish_destinations: destinations(),
            },
        ),
        Err(NativeSourceFeedbackError::DurableLsnBehindBatch {
            batch_last_commit_lsn: "0/16B6C60".to_string(),
            durable_publish_lsn: "0/16B6C50".to_string(),
        })
    );
}

#[test]
fn source_feedback_rejects_wrong_publish_destination_proof() {
    let batch = drain_batch();

    assert_eq!(
        native_source_feedback_decision(
            &batch,
            NativeSourceFeedbackProof {
                durable_publish_lsn: "0/16B6C60".to_string(),
                published_frame_checksums: vec![7, 8],
                expected_publish_destinations: destinations(),
                durable_publish_destinations: vec![
                    destination("orders", 0),
                    destination("orders-retry", 1),
                ],
            },
        ),
        Err(NativeSourceFeedbackError::PublishDestinationMismatch)
    );
}

fn drain_batch() -> NativeHandoffDrainBatch {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");
    native_handoff_drain_batch(
        &config,
        vec![
            frame("tx-1", "0/16B6C50", 1024, 7),
            frame("tx-2", "0/16B6C60", 512, 8),
        ],
    )
    .expect("drain batch")
}

fn destinations() -> Vec<NativePublishDestination> {
    vec![destination("orders", 0), destination("orders", 1)]
}

fn destination(topic: &str, partition: i32) -> NativePublishDestination {
    NativePublishDestination {
        topic: topic.to_string(),
        partition,
    }
}

fn frame(
    transaction_id: &str,
    commit_lsn: &str,
    payload_bytes: usize,
    checksum: u64,
) -> NativeHandoffFrame {
    native_committed_transaction_frame(
        "source-a",
        "orders",
        transaction_id,
        commit_lsn,
        payload_bytes,
        checksum,
    )
    .expect("handoff frame")
}
