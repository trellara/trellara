use super::helpers::{commit_marker, partition_plan};
use super::*;

#[test]
fn partitioned_messages_reject_invalid_envelope_schema_evidence() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let chunk = plan.chunks.first().expect("chunk");
    let marker = commit_marker(&plan);

    let mut invalid_envelope = envelope;
    invalid_envelope.schema_versions = vec![RelationSchemaVersion {
        relation: None,
        version: 7,
    }];
    invalid_envelope.finalize_checksum();

    assert_invalid_schema_evidence(StreamMessage::transaction_manifest(
        &invalid_envelope,
        &plan.manifest,
    ));
    assert_invalid_schema_evidence(StreamMessage::partition_chunk(&invalid_envelope, chunk));
    assert_invalid_schema_evidence(StreamMessage::strict_chunk(&invalid_envelope, chunk));
    assert_invalid_schema_evidence(StreamMessage::commit_marker(&invalid_envelope, &marker));
}

#[test]
fn partitioned_messages_reject_stale_envelope_checksum() {
    let envelope = sample_envelope();
    let plan = partition_plan();

    let mut stale_envelope = envelope;
    stale_envelope.dataset_id = "tampered".to_string();

    assert!(matches!(
        StreamMessage::transaction_manifest(&stale_envelope, &plan.manifest),
        Err(StreamError::Protocol(
            ProtocolError::ChecksumMismatch { .. }
        ))
    ));
}

#[test]
fn commit_marker_messages_reject_invalid_marker_protocol_fields() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let mut marker = commit_marker(&plan);
    marker.manifest_checksum = 0;

    assert!(matches!(
        StreamMessage::commit_marker(&envelope, &marker),
        Err(StreamError::Protocol(
            ProtocolError::InvalidCommitMarkerField {
                field: "manifest_checksum",
                ..
            }
        ))
    ));
}

fn assert_invalid_schema_evidence(result: Result<StreamMessage>) {
    assert!(matches!(
        result,
        Err(StreamError::Protocol(
            ProtocolError::InvalidSchemaVersionEvidence { reason, .. }
        )) if reason == "relation metadata is required"
    ));
}
