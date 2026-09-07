use super::*;

#[test]
fn lake_spark_consumption_requires_release_and_source_ack_boundary() {
    let completeness = r#"{
        "dataset_id": "retail-sales",
        "source_rows": [
            {"source_id": "local-source", "state": "complete"}
        ],
        "verification_status": "match",
        "spark_consumption_allowed": true,
        "source_counts_match": true,
        "stream_required_source_count": 1,
        "stream_complete_source_count": 1,
        "stream_missing_source_count": 0,
        "stream_quarantined_source_count": 0,
        "lake_required_source_count": 1,
        "lake_complete_source_count": 1,
        "lake_missing_source_count": 0,
        "lake_quarantined_source_count": 0,
        "checksum_rollup_match": true,
        "stream_checksum_rollup": 991,
        "lake_checksum_rollup": 991,
        "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
        "spark_consumption_gate": "released: stream and lake proofs match",
        "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
    }"#;

    for marker in live_evidence_expected_markers("lake_spark_consumption") {
        assert!(
            live_evidence_marker_present("lake_spark_consumption", &marker, completeness),
            "expected marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source dataset identity",
        r#"{
            "source_rows": [
                {"source_id": "local-source", "state": "complete"}
            ],
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source dataset identity",
        r#"{
            "dataset_id": "unknown",
            "source_rows": [
                {"source_id": "local-source", "state": "complete"}
            ],
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source dataset identity",
        r#"{
            "dataset_id": "retail-sales",
            "source_rows": [
                {"source_id": "", "state": "complete"}
            ],
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "spark_consumption_allowed true",
        r#"{"verification_status":"match","spark_consumption_allowed":false}"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "verification_status match",
        r#"{"verification_status":"mismatch","spark_consumption_allowed":true}"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "released spark consumption gate",
        r#"{
            "verification_status": "mismatch",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "released spark consumption gate",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": false,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "released spark consumption gate",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "blocked: complete_with_gaps requires explicit acceptance",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "released spark consumption gate",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released by operator",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source ack boundary",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match"
        }"#
    ));
    assert!(live_evidence_marker_present(
        "lake_spark_consumption",
        "source ack boundary",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "committer_topology": {
                "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source ack boundary",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "operator reviewed source ack behavior"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source ack boundary",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "spark consumption contract",
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "spark_consumption_contract": "operator_release",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "spark_consumption_allowed true",
        "spark_consumption_allowed=true"
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source count agreement",
        r#"{
            "source_counts_match": true,
            "stream_required_source_count": 1,
            "lake_required_source_count": 2,
            "stream_complete_source_count": 1,
            "lake_complete_source_count": 1,
            "stream_missing_source_count": 0,
            "lake_missing_source_count": 0,
            "stream_quarantined_source_count": 0,
            "lake_quarantined_source_count": 0
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source count agreement",
        &completeness.replace(r#""state": "complete""#, r#""state": "missing""#)
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source count agreement",
        &completeness.replace(r#""state": "complete""#, r#""state": "unknown""#)
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "source count agreement",
        r#"{
            "source_counts_match": false,
            "stream_required_source_count": 1,
            "lake_required_source_count": 1,
            "stream_complete_source_count": 1,
            "lake_complete_source_count": 1,
            "stream_missing_source_count": 0,
            "lake_missing_source_count": 0,
            "stream_quarantined_source_count": 0,
            "lake_quarantined_source_count": 0
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "checksum rollup agreement",
        r#"{
            "checksum_rollup_match": true,
            "stream_checksum_rollup": 991,
            "lake_checksum_rollup": 992
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "lake_spark_consumption",
        "checksum rollup agreement",
        r#"{
            "checksum_rollup_match": false,
            "stream_checksum_rollup": 991,
            "lake_checksum_rollup": 991
        }"#
    ));
}
