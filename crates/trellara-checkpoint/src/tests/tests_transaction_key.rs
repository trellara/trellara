use super::*;

#[test]
fn transaction_key_round_trips_through_boundary_contract() {
    let key = TransactionKey::from_envelope(&sample_envelope());
    let boundary = key.to_boundary_key().expect("boundary key");

    assert_eq!(boundary.to_string(), "source_a:retail:sales:tx-1:0/16B6C50");
    assert_eq!(TransactionKey::from_boundary_key(boundary), key);
}

#[test]
fn checked_transaction_key_from_envelope_canonicalizes_commit_lsn() {
    let mut envelope = sample_envelope();
    envelope.commit_lsn = "00000000/016B6C50".to_string();
    envelope.finalize_checksum();

    let key = TransactionKey::try_from_envelope(&envelope).expect("transaction key");

    assert_eq!(key.commit_lsn, "0/16B6C50");
}

#[test]
fn transaction_key_validation_preserves_commit_lsn_context() {
    let mut key = TransactionKey::from_envelope(&sample_envelope());
    key.commit_lsn = "0/0".to_string();

    let error = key.validate().expect_err("zero commit lsn rejected");

    assert!(error.to_string().contains("commit_lsn"));
    assert!(error.to_string().contains("LSN must be greater than zero"));
}
