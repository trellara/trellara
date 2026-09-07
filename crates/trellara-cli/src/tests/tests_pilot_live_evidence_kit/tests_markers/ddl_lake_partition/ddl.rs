use super::*;

mod ack_commands;
mod ack_evidence;
mod identity_policy;
mod release_evidence;
mod release_gate;
mod replay_proof;

const ARTIFACT: &str = "ddl_release_proof";

#[test]
fn ddl_release_proof_requires_all_expected_markers() {
    assert_eq!(
        live_evidence_artifact_name(ARTIFACT),
        "ddl-release-proof.json"
    );
    for marker in live_evidence_expected_markers(ARTIFACT) {
        assert_marker_present(&marker, valid_release_proof());
    }
}

#[test]
fn ddl_release_proof_rejects_vague_cdc_transaction_boundary() {
    assert!(!live_evidence_marker_present(
        ARTIFACT,
        "cdc_transaction_boundary",
        r#"{
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier"
        }"#
    ));
    assert!(!live_evidence_marker_present(
        ARTIFACT,
        "cdc_transaction_boundary",
        r#"{
            "release_summary": {
                "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until operator release"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        ARTIFACT,
        "cdc_transaction_boundary",
        "cdc_transaction_boundary: source commit LSN is the DDL barrier"
    ));
    assert!(live_evidence_marker_present(
        ARTIFACT,
        "cdc_transaction_boundary",
        "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn"
    ));
}

fn assert_marker_present(marker: &str, proof: &str) {
    assert!(
        live_evidence_marker_present(ARTIFACT, marker, proof),
        "expected marker {marker}"
    );
}

fn valid_release_proof() -> &'static str {
    r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "release_decision": {
            "release_dml": true,
            "blocker_codes": []
        },
        "release_gates": [
            {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
        ],
        "barrier_id": "ddl-barrier-abc",
        "barrier_lsn": "0/16B8000",
        "schema_version": "schema-v2",
        "required_sinks": ["raw_cdc_lake"],
        "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
        "propagation_boundary": "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks",
        "propagation_decisions": ["auto_apply:1", "manual_review:0", "unsupported:0", "target_ack_required:1"],
        "propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "ack_commands": [
            "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        ],
        "ack_evidence": [
            {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
        ],
        "release_evidence": [
            "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "1/1 required sink ACKs accepted",
            "post-DDL DML release_dml=true blocker_codes=none"
        ],
        "ddl_dml_replay_proof": {
            "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "target_ack_lsn": "00000000/016B8000",
            "dml_commit_lsn": "0/16B8000",
            "dml_decision": "Applied",
            "ddl_applied_statements": 1,
            "dml_applied_changes": 1,
            "schema_version": "schema-v2",
            "release_gate": "post_ddl_dml_release",
            "target_transaction_boundary": "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release",
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "proof_steps": [
                "target_postgres_recorded_ddl_ack",
                "release_decision_allowed_post_ddl_dml",
                "dml_replay_applied_changes_positive",
                "dml_replay_commit_lsn_matches_ddl_barrier_lsn"
            ]
        },
        "release_blocker_codes": []
    }"#
}
