use super::*;

#[test]
fn transaction_inspector_distinguishes_strict_chunk_manifest_boundary() {
    let mut envelope = inspect_envelope();
    envelope.manifest.as_mut().expect("manifest").boundary_mode =
        trellara_protocol::ManifestBoundaryMode::StrictChunkedTransactionOrder as i32;
    envelope.finalize_checksum();

    let summary = TransactionInspectSummary::from_envelope(&envelope);

    assert_eq!(
        summary.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    assert_eq!(
        summary.transaction_boundary.mode,
        "strict_chunked_transaction_order"
    );
    assert!(summary
        .transaction_boundary
        .guarantee
        .contains("strict chunk manifest and commit marker"));
    assert!(summary
        .transaction_boundary
        .visibility_contract
        .contains("every strict chunk before visibility"));
    assert_eq!(
        summary
            .partition_manifest
            .as_ref()
            .expect("manifest")
            .boundary_mode,
        "strict_chunked_transaction_order"
    );

    let output = render_transaction_inspect_summary(&summary, TransactionInspectOutputFormat::Text)
        .expect("transaction inspect text");
    assert!(output.contains("strict_chunk_manifest:"));
    assert!(output.contains(
            "boundary_mode=strict_chunked_transaction_order global_event_count=3 participating_chunk_count=2"
        ));
    assert!(output.contains("- chunk=0 events=1 total_order=1..1"));
}
