use super::*;

#[test]
fn handoff_frame_preserves_committed_transaction_boundary() {
    let frame = native_committed_transaction_frame(
        "source-a",
        "orders",
        "tx-1",
        "00000000/016B6C50",
        4096,
        42,
    )
    .expect("handoff frame");

    assert_eq!(frame.kind, NativeHandoffFrameKind::CommittedTransaction);
    assert_eq!(frame.commit_lsn, "0/16B6C50");
    assert_eq!(frame.payload_bytes, 4096);
    assert_eq!(frame.envelope_checksum, 42);
    assert_eq!(
        frame.source_acknowledgement,
        SOURCE_ACKNOWLEDGEMENT_CONTRACT
    );
}

#[test]
fn handoff_frame_rejects_ambiguous_boundary_identity() {
    let error =
        native_committed_transaction_frame(" source-a ", "orders", "tx-1", "0/16B6C50", 1, 42)
            .expect_err("padded source id");

    assert!(matches!(
        error,
        NativeHandoffError::InvalidBoundaryField {
            field: "source_id",
            reason,
        } if reason.contains("surrounding whitespace")
    ));
}

#[test]
fn handoff_frame_rejects_unbounded_or_unproven_payloads() {
    assert_eq!(
        native_committed_transaction_frame("source-a", "orders", "tx-1", "0/16B6C50", 0, 42),
        Err(NativeHandoffError::EmptyPayload)
    );
    assert_eq!(
        native_committed_transaction_frame(
            "source-a",
            "orders",
            "tx-1",
            "0/16B6C50",
            MAX_HANDOFF_PAYLOAD_BYTES + 1,
            42,
        ),
        Err(NativeHandoffError::PayloadTooLarge {
            payload_bytes: MAX_HANDOFF_PAYLOAD_BYTES + 1,
            max_payload_bytes: MAX_HANDOFF_PAYLOAD_BYTES,
        })
    );
    assert_eq!(
        native_committed_transaction_frame("source-a", "orders", "tx-1", "0/16B6C50", 1, 0),
        Err(NativeHandoffError::MissingEnvelopeChecksum)
    );
}
