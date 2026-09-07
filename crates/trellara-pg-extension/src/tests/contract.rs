use super::*;

#[test]
fn capture_contract_preserves_existing_correctness_boundaries() {
    let contract =
        native_capture_contract("source-a", "orders", 17).expect("PostgreSQL 17 is supported");

    assert_eq!(
        contract.protocol_version,
        trellara_protocol::PROTOCOL_VERSION
    );
    assert_eq!(
        contract.transaction_visibility,
        "committed_transactions_only"
    );
    assert_eq!(
        contract.source_acknowledgement,
        "only_after_durable_stream_publish"
    );
    assert!(!contract.broker_io_inside_postgres);
    assert!(!contract.data_plane_ready);
}

#[test]
fn capture_contract_rejects_ambiguous_identity() {
    assert_eq!(
        native_capture_contract(" ", "orders", 18),
        Err(CaptureContractError::BlankSourceId)
    );
    assert_eq!(
        native_capture_contract("source-a", "", 18),
        Err(CaptureContractError::BlankDatasetId)
    );
    assert_eq!(
        native_capture_contract(" source-a ", "orders", 18),
        Err(CaptureContractError::PaddedSourceId)
    );
    assert_eq!(
        native_capture_contract("source-a", " orders ", 18),
        Err(CaptureContractError::PaddedDatasetId)
    );
}
