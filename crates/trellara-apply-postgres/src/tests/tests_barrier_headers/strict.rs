use super::*;

#[test]
fn strict_header_context_rejects_ddl_event_count_mismatch_directly() {
    let envelope = envelope("tx-strict-header-count", "0/16B6FC0");
    let mut message = StreamMessage::strict_transaction(&envelope).expect("strict message");
    replace_header(&mut message, "trellara.ddl_event_count", "1");

    let error = crate::barrier::validate_strict_header_context(&message, &envelope)
        .expect_err("DDL count mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "ddl_event_count",
            ..
        }
    ));

    message.headers.clear();
    crate::barrier::validate_strict_header_context(&message, &envelope)
        .expect("optional strict headers can be absent");
}

#[test]
fn strict_header_context_requires_ddl_proof_headers_for_ddl_transactions() {
    let envelope = ddl_envelope();
    let mut message = StreamMessage::strict_transaction(&envelope).expect("strict message");
    remove_header(&mut message, "trellara.ddl_propagation_policy_sha256");

    let error = crate::barrier::validate_strict_header_context(&message, &envelope)
        .expect_err("missing DDL proof header");

    assert!(matches!(
        error,
        ApplyWorkerError::MissingHeader("trellara.ddl_propagation_policy_sha256")
    ));
}

#[test]
fn strict_header_context_rejects_mismatched_ddl_proof_headers() {
    let envelope = ddl_envelope();
    let mut message = StreamMessage::strict_transaction(&envelope).expect("strict message");
    replace_header(&mut message, "trellara.ddl_target_ack_required", "false");

    let error = crate::barrier::validate_strict_header_context(&message, &envelope)
        .expect_err("DDL proof mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "ddl_target_ack_required",
            ..
        }
    ));
}

fn ddl_envelope() -> TransactionEnvelope {
    let mut envelope = envelope("tx-strict-ddl-header-proof", "0/16B7000");
    envelope.ddl_events = vec![DdlEvent::manual_review(
        "tx-strict-ddl-header-proof",
        2,
        DdlOperation::AddColumn,
        relation(),
        "ALTER TABLE public.orders ADD COLUMN note text",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}
