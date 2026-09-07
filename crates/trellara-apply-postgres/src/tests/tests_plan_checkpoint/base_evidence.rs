use super::*;

#[test]
fn checkpoint_evidence_accepts_strict_envelope_without_manifest() {
    let envelope = envelope("tx-checkpoint-strict", "0/16B6C50");

    validate_apply_checkpoint_evidence(&envelope).expect("valid checkpoint evidence");
}

#[test]
fn checkpoint_evidence_rejects_missing_flow_identity() {
    let mut envelope = envelope("tx-missing-source", "0/16B6C50");
    envelope.source_id.clear();

    let error =
        validate_apply_checkpoint_evidence(&envelope).expect_err("missing source id rejected");

    assert!(matches!(
        error,
        ApplyError::MissingCheckpointEvidence { field: "source_id" }
    ));
}

#[test]
fn checkpoint_evidence_rejects_invalid_commit_lsn_before_sql() {
    let envelope = envelope("tx-invalid-lsn", "not-an-lsn");

    let error =
        validate_apply_checkpoint_evidence(&envelope).expect_err("invalid commit lsn rejected");

    assert!(matches!(
        error,
        ApplyError::Protocol(ProtocolError::InvalidLsn { .. })
    ));
}

#[test]
fn checkpoint_evidence_rejects_zero_commit_lsn_before_sql() {
    let envelope = envelope("tx-zero-lsn", "0/0");

    let error = validate_apply_checkpoint_evidence(&envelope).expect_err("zero lsn rejected");

    assert!(matches!(
        error,
        ApplyError::MissingCheckpointEvidence {
            field: "commit_lsn"
        }
    ));
}

#[test]
fn checkpoint_evidence_rejects_duplicate_transaction_event_order_before_sql() {
    let mut envelope = envelope("tx-duplicate-order", "0/16B6C50");
    let mut duplicate = envelope.changes[0].clone();
    duplicate.idempotency_key = idempotency_key("source", "0/16B6C50", "tx-duplicate-order", 99);
    envelope.changes.push(duplicate);
    envelope.finalize_checksum();

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("duplicate transaction order rejected");

    assert!(matches!(
        error,
        ApplyError::Protocol(ProtocolError::DuplicateTransactionEventOrder { total_order: 1 })
    ));
}
