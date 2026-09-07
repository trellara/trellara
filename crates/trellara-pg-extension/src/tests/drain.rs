use super::*;

#[test]
fn handoff_drain_batch_preserves_worker_to_relay_ack_contract() {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");
    let frames = vec![
        frame("tx-1", "0/16B6C50", 1024, 7),
        frame("tx-2", "0/16B6C60", 512, 8),
    ];

    let batch = native_handoff_drain_batch(&config, frames.clone()).expect("drain batch");

    assert_eq!(batch.contract, RELAY_HANDOFF_CONTRACT);
    assert_eq!(batch.frames, frames);
    assert_eq!(batch.frame_count, 2);
    assert_eq!(batch.payload_bytes, 1536);
    assert_eq!(batch.first_commit_lsn, "0/16B6C50");
    assert_eq!(batch.last_commit_lsn, "0/16B6C60");
    assert_eq!(
        batch.source_acknowledgement,
        SOURCE_ACKNOWLEDGEMENT_CONTRACT
    );
}

#[test]
fn handoff_drain_batch_rejects_unbounded_or_empty_drains() {
    let config = native_handoff_queue_config(1, 2048, 2048).expect("queue config");

    assert_eq!(
        native_handoff_drain_batch(&config, Vec::new()),
        Err(NativeHandoffDrainError::EmptyBatch)
    );
    assert_eq!(
        native_handoff_drain_batch(
            &config,
            vec![
                frame("tx-1", "0/16B6C50", 1024, 7),
                frame("tx-2", "0/16B6C60", 512, 8)
            ],
        ),
        Err(NativeHandoffDrainError::BatchExceedsQueueCapacity {
            frame_count: 2,
            capacity_frames: 1,
        })
    );
}

#[test]
fn handoff_drain_batch_rejects_reordered_or_duplicate_boundaries() {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");

    assert_eq!(
        native_handoff_drain_batch(
            &config,
            vec![
                frame("tx-2", "0/16B6C60", 512, 8),
                frame("tx-1", "0/16B6C50", 1024, 7)
            ],
        ),
        Err(NativeHandoffDrainError::CommitLsnOrderRegression {
            previous_commit_lsn: "0/16B6C60".to_string(),
            current_commit_lsn: "0/16B6C50".to_string(),
        })
    );

    let duplicate = frame("tx-1", "0/16B6C50", 1024, 7);
    assert_eq!(
        native_handoff_drain_batch(&config, vec![duplicate.clone(), duplicate]),
        Err(NativeHandoffDrainError::DuplicateTransactionBoundary {
            boundary: "source-a:orders:tx-1:0/16B6C50".to_string(),
        })
    );
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
