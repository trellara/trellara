use super::fixtures::{default_envelope, envelope_with};
use super::*;

#[test]
fn checked_encoding_rejects_zero_commit_lsn() {
    let envelope = envelope_with("tx-1", "0/16B6B00", "0/0", vec![sample_change(1)]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidLsn { .. })
    ));
}

#[test]
fn checked_decoding_rejects_missing_identity_with_valid_checksum() {
    let mut envelope = default_envelope();
    envelope.source_id.clear();
    envelope.finalize_checksum();

    assert!(matches!(
        TransactionEnvelope::decode_checked(&envelope.encode_to_vec()),
        Err(ProtocolError::MissingEnvelopeField { field: "source_id" })
    ));
}

#[test]
fn checked_encoding_rejects_identity_with_surrounding_whitespace() {
    for field in ["source_id", "database_id", "dataset_id", "transaction_id"] {
        let mut envelope = default_envelope();
        set_spaced_identity(&mut envelope, field);
        envelope.finalize_checksum();

        assert!(matches!(
            envelope.encode_checked(),
            Err(ProtocolError::InvalidEnvelopeField {
                field: actual,
                reason,
            }) if actual == field && reason.contains("surrounding whitespace")
        ));
    }
}

#[test]
fn checked_encoding_rejects_missing_database_identity() {
    let mut envelope = default_envelope();
    envelope.database_id.clear();
    envelope.finalize_checksum();

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::MissingEnvelopeField {
            field: "database_id",
        })
    ));
}

#[test]
fn checked_encoding_rejects_non_positive_commit_timestamp() {
    for timestamp in [0, -1] {
        let mut envelope = default_envelope();
        envelope.commit_timestamp_ms = timestamp;
        envelope.finalize_checksum();

        assert!(matches!(
            envelope.encode_checked(),
            Err(ProtocolError::InvalidEnvelopeField {
                field: "commit_timestamp_ms",
                reason,
            }) if reason.contains("greater than zero")
        ));
    }
}

fn set_spaced_identity(envelope: &mut TransactionEnvelope, field: &str) {
    match field {
        "source_id" => envelope.source_id = " source-a ".to_string(),
        "database_id" => envelope.database_id = " retail ".to_string(),
        "dataset_id" => envelope.dataset_id = " sales ".to_string(),
        "transaction_id" => envelope.transaction_id = " tx-1 ".to_string(),
        _ => unreachable!("test field"),
    }
}

#[test]
fn checked_encoding_rejects_begin_lsn_after_commit_lsn() {
    let envelope = envelope_with("tx-1", "0/16B6C51", "0/16B6C50", vec![sample_change(1)]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::EnvelopeLsnOrder { .. })
    ));
}

#[test]
fn checked_encoding_rejects_change_transaction_mismatch() {
    let mut change = sample_change(1);
    change.transaction_id = "tx-other".to_string();
    let envelope = envelope_with("tx-1", "0/16B6B00", "0/16B6C50", vec![change]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::ChangeTransactionMismatch { .. })
    ));
}

#[test]
fn checked_encoding_rejects_missing_change_idempotency_key() {
    let mut envelope = default_envelope();
    envelope.changes[0].idempotency_key = " ".to_string();
    envelope.finalize_checksum();

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidChangeField(error))
            if error.total_order == 1
                && error.field == "idempotency_key"
                && error.reason.contains("empty")
    ));
}

#[test]
fn checked_encoding_rejects_padded_change_idempotency_key() {
    let mut envelope = default_envelope();
    envelope.changes[0].idempotency_key = format!(" {} ", envelope.changes[0].idempotency_key);
    envelope.finalize_checksum();

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidChangeField(error))
            if error.total_order == 1
                && error.field == "idempotency_key"
                && error.reason.contains("surrounding whitespace")
    ));
}

#[test]
fn checked_encoding_rejects_duplicate_total_order() {
    let envelope = envelope_with(
        "tx-1",
        "0/16B6B00",
        "0/16B6C50",
        vec![sample_change(1), sample_change(1)],
    );

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order: 1 })
    ));
}

#[test]
fn checked_encoding_allows_empty_begin_lsn_for_reconstructed_envelopes() {
    let envelope = envelope_with("tx-1", "", "0/16B6C50", vec![sample_change(1)]);

    envelope
        .encode_checked()
        .expect("encode reconstructed envelope");
}
