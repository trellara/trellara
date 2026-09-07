use super::*;

#[test]
fn ddl_release_proof_rejects_incomplete_release_evidence() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{"release_decision":{"release_dml":true},"release_evidence":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "release_decision": {"release_dml": true},
            "release_evidence": ["operator reviewed the release proof"]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "4/4 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        &valid_release_proof().replace(
            r#""ack_evidence": [
            {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
        ],"#,
            r#""ack_evidence": [],"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        &valid_release_proof().replace(r#""accepted": true"#, r#""accepted": false"#)
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_lsn": "0/16B9000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "4/4 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v3",
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "4/4 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_id": "ddl-barrier-new",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "4/4 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true, "blocker_codes": ["pending_required_sink_ack"]},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_evidence",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "release_decision": {"release_dml": false, "blocker_codes": []},
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ]
        }"#
    ));
}
