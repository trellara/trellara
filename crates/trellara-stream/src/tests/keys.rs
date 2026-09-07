use super::*;
use trellara_protocol::ProtocolError;

#[test]
fn strict_transaction_key_uses_boundary_contract() {
    let key = strict_transaction_key(&sample_envelope()).expect("strict key");

    assert_eq!(key, "source_a:retail:sales:tx-1:0/16B6C50");
}

#[test]
fn partition_chunk_key_extends_boundary_contract_with_partition_id() {
    let key = partition_chunk_key(&sample_envelope(), 7).expect("partition key");

    assert_eq!(key, "source_a:retail:sales:tx-1:0/16B6C50:7");
}

#[test]
fn stream_key_canonicalizes_lsn_text() {
    let mut envelope = sample_envelope();
    envelope.commit_lsn = "00000000/016B6C50".to_string();
    envelope.finalize_checksum();

    let key = strict_transaction_key(&envelope).expect("canonical key");

    assert_eq!(key, "source_a:retail:sales:tx-1:0/16B6C50");
}

#[test]
fn stream_key_rejects_missing_transaction_identity() {
    let mut envelope = sample_envelope();
    envelope.transaction_id.clear();
    envelope.finalize_checksum();

    let error = strict_transaction_key(&envelope).expect_err("invalid boundary key");

    assert!(matches!(
        error,
        StreamError::Protocol(ProtocolError::MissingEnvelopeField {
            field: "transaction_id"
        })
    ));
}
