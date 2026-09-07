use super::*;

#[test]
fn handoff_queue_config_sets_bounded_backpressure_contract() {
    let config = native_handoff_queue_config(64, 1024, 16 * 1024).expect("queue config");

    assert_eq!(config.capacity_frames, 64);
    assert_eq!(config.max_frame_payload_bytes, 1024);
    assert_eq!(config.max_buffered_payload_bytes, 16 * 1024);
    assert_eq!(
        config.overflow_policy,
        "block_source_backend_until_relay_drains"
    );
}

#[test]
fn handoff_queue_rejects_unbounded_capacity_config() {
    assert_eq!(
        native_handoff_queue_config(0, 1024, 1024),
        Err(NativeHandoffQueueError::InvalidFrameCapacity)
    );
    assert_eq!(
        native_handoff_queue_config(MAX_HANDOFF_QUEUE_FRAMES + 1, 1024, 1024),
        Err(NativeHandoffQueueError::InvalidFrameCapacity)
    );
    assert_eq!(
        native_handoff_queue_config(1, MAX_HANDOFF_PAYLOAD_BYTES + 1, 1024),
        Err(NativeHandoffQueueError::InvalidFramePayloadLimit)
    );
}

#[test]
fn handoff_queue_admission_blocks_overflow_without_source_ack() {
    let config = native_handoff_queue_config(2, 1024, 2048).expect("queue config");
    let frame =
        native_committed_transaction_frame("source-a", "orders", "tx-1", "0/16B6C50", 1024, 42)
            .expect("handoff frame");

    let admission = native_handoff_queue_admission(&config, 1, 512, &frame).expect("admission");
    assert_eq!(admission.queued_frames_after, 2);
    assert_eq!(admission.queued_payload_bytes_after, 1536);
    assert_eq!(
        admission.source_acknowledgement,
        SOURCE_ACKNOWLEDGEMENT_CONTRACT
    );

    assert_eq!(
        native_handoff_queue_admission(&config, 2, 1024, &frame),
        Err(NativeHandoffQueueError::FrameCapacityExceeded {
            queued_frames: 2,
            capacity_frames: 2,
        })
    );
    assert!(matches!(
        native_handoff_queue_admission(&config, 1, 1536, &frame),
        Err(NativeHandoffQueueError::BufferedPayloadCapacityExceeded { .. })
    ));
}
