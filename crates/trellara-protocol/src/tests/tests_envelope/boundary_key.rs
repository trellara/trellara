use super::fixtures::default_envelope;
use super::*;

#[test]
fn boundary_key_uses_exact_transaction_identity() {
    let envelope = default_envelope();

    let key = envelope.boundary_key().expect("boundary key");

    assert_eq!(key.source_id, "source-a");
    assert_eq!(key.database_id, "retail");
    assert_eq!(key.dataset_id, "sales");
    assert_eq!(key.transaction_id, "tx-1");
    assert_eq!(key.commit_lsn, "0/16B6C50");
    assert_eq!(key.to_string(), "source-a:retail:sales:tx-1:0/16B6C50");
}

#[test]
fn boundary_key_canonicalizes_equivalent_lsn_text() {
    let key = TransactionBoundaryKey::new_with_database(
        "source-a",
        "retail",
        "sales",
        "tx-1",
        "00000000/016B6C50",
    )
    .expect("boundary key");

    assert_eq!(key.commit_lsn, "0/16B6C50");
}

#[test]
fn boundary_key_separates_same_lsn_transaction_across_databases() {
    let retail = TransactionBoundaryKey::new_with_database(
        "source-a",
        "retail",
        "sales",
        "tx-1",
        "0/16B6C50",
    )
    .expect("retail boundary");
    let analytics = TransactionBoundaryKey::new_with_database(
        "source-a",
        "analytics",
        "sales",
        "tx-1",
        "0/16B6C50",
    )
    .expect("analytics boundary");

    assert_ne!(retail, analytics);
    assert_ne!(retail.to_string(), analytics.to_string());
}

#[test]
fn boundary_key_rejects_partial_transaction_identity() {
    let error =
        TransactionBoundaryKey::new_with_database("source-a", "retail", "sales", " ", "0/16B6C50")
            .expect_err("missing transaction id");

    assert!(matches!(
        error,
        ProtocolError::MissingEnvelopeField {
            field: "transaction_id"
        }
    ));
}

#[test]
fn boundary_key_rejects_padded_transaction_identity() {
    for (field, source_id, database_id, dataset_id, transaction_id, commit_lsn) in [
        (
            "source_id",
            " source-a ",
            "retail",
            "sales",
            "tx-1",
            "0/16B6C50",
        ),
        (
            "database_id",
            "source-a",
            " retail ",
            "sales",
            "tx-1",
            "0/16B6C50",
        ),
        (
            "dataset_id",
            "source-a",
            "retail",
            " sales ",
            "tx-1",
            "0/16B6C50",
        ),
        (
            "transaction_id",
            "source-a",
            "retail",
            "sales",
            " tx-1 ",
            "0/16B6C50",
        ),
        (
            "commit_lsn",
            "source-a",
            "retail",
            "sales",
            "tx-1",
            " 0/16B6C50 ",
        ),
    ] {
        let error = TransactionBoundaryKey::new_with_database(
            source_id,
            database_id,
            dataset_id,
            transaction_id,
            commit_lsn,
        )
        .expect_err("padded boundary field");

        assert!(matches!(
            error,
            ProtocolError::InvalidEnvelopeField {
                field: actual,
                reason,
            } if actual == field && reason.contains("surrounding whitespace")
        ));
    }
}

#[test]
fn boundary_key_rejects_zero_commit_lsn() {
    let error =
        TransactionBoundaryKey::new_with_database("source-a", "retail", "sales", "tx-1", "0/0")
            .expect_err("zero LSN");

    assert!(matches!(error, ProtocolError::InvalidLsn { .. }));
}

#[test]
fn boundary_key_derives_event_idempotency_key() {
    let key = default_envelope().boundary_key().expect("boundary key");

    assert_eq!(
        key.event_idempotency_key(42),
        idempotency_key("source-a", "0/16B6C50", "tx-1", 42)
    );
}
