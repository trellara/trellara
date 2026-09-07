use super::*;

#[test]
fn ddl_release_proof_rejects_incomplete_ack_evidence() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{"release_decision":{"release_dml":true},"ack_evidence":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{"release_decision":{"release_dml":true},"ack_evidence":[{}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{"release_decision":{"release_dml":true},"ack_evidence":[{"sink":"","source_ack_lsn":"0/16B6C50"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": " raw_cdc_lake ", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{"release_decision":{"release_dml":true},"ack_evidence":[{"sink":"raw_cdc_lake","source_ack_lsn":"not-a-lsn"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "source_id": "local-source",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"source_id": "shadow-source", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "database_id": "retail",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"database_id": "analytics", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"dataset_id": "warehouse-shadow", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "source_ack_lsn": "0/16B6C50", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "accepted": false}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000"}
            ]
        }"#
    ));
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v1 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-old", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v1", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "target_postgres", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true},
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true},
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake", "raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
}
