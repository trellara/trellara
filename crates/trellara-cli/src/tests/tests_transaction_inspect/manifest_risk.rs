use super::*;

#[test]
fn transaction_inspector_marks_manifest_count_mismatch_at_risk() {
    let mut envelope = inspect_envelope();
    envelope
        .manifest
        .as_mut()
        .expect("manifest")
        .global_event_count = 4;
    envelope.finalize_checksum();

    let summary = TransactionInspectSummary::from_envelope(&envelope);

    assert_eq!(
        summary.transaction_boundary.status,
        TransactionBoundaryStatus::AtRisk
    );
    assert_eq!(
        summary.transaction_boundary.checksum_status,
        ChecksumStatus::Match
    );
    assert_eq!(
        summary.transaction_boundary.global_event_count_matches,
        Some(false)
    );
    assert_eq!(
        summary.transaction_boundary.partition_event_count_matches,
        Some(true)
    );
    assert!(!summary.transaction_boundary.manifest_valid);
    assert!(summary
        .transaction_boundary
        .manifest_validation_error
        .as_deref()
        .is_some_and(|error| error.contains("event count")));
}

#[test]
fn transaction_inspector_marks_invalid_manifest_boundary_mode_at_risk() {
    let mut envelope = inspect_envelope();
    envelope.manifest.as_mut().expect("manifest").boundary_mode =
        trellara_protocol::ManifestBoundaryMode::Unspecified as i32;
    envelope.finalize_checksum();

    let summary = TransactionInspectSummary::from_envelope(&envelope);

    assert_eq!(
        summary.transaction_boundary.status,
        TransactionBoundaryStatus::AtRisk
    );
    assert_eq!(
        summary.transaction_boundary.checksum_status,
        ChecksumStatus::Match
    );
    assert_eq!(
        summary.transaction_boundary.global_event_count_matches,
        Some(true)
    );
    assert_eq!(
        summary.transaction_boundary.partition_event_count_matches,
        Some(true)
    );
    assert!(!summary.transaction_boundary.manifest_valid);
    assert!(summary
        .transaction_boundary
        .manifest_validation_error
        .as_deref()
        .is_some_and(|error| error.contains("boundary_mode")));
    assert_eq!(
        summary.transaction_boundary.mode,
        "manifest_barrier_transaction"
    );
}

#[test]
fn transaction_inspector_text_renders_manifest_validation_error() {
    let mut envelope = inspect_envelope();
    envelope.manifest.as_mut().expect("manifest").boundary_mode =
        trellara_protocol::ManifestBoundaryMode::Unspecified as i32;
    envelope.finalize_checksum();

    let output = render_transaction_inspect_summary(
        &TransactionInspectSummary::from_envelope(&envelope),
        TransactionInspectOutputFormat::Text,
    )
    .expect("transaction inspect text");

    assert!(output.contains("manifest_valid=false"));
    assert!(output.contains("manifest_validation_error="));
    assert!(output.contains("boundary_mode"));
}
