use super::*;

#[test]
fn ddl_release_proof_rejects_mismatched_replay_proof() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "cdc_transaction_boundary",
        r#"{"release_decision":{"release_dml":true},"ack_evidence":[{}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        &valid_release_proof().replace(r#""release_dml": true"#, r#""release_dml": false"#)
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        &valid_release_proof().replace(
            r#""blocker_codes": []"#,
            r#""blocker_codes": ["pending_required_sink_ack"]"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B9000",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "source_id": "local-source",
                "database_id": "retail",
                "dataset_id": "retail-sales",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "ddl_applied_statements": 0,
                "dml_decision": "Applied",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "source_id": "shadow-source",
                "database_id": "retail",
                "dataset_id": "retail-sales",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "ddl_applied_statements": 1,
                "dml_decision": "Applied",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "source_id": "local-source",
                "database_id": "analytics",
                "dataset_id": "retail-sales",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "ddl_applied_statements": 1,
                "dml_decision": "Applied",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "source_id": "local-source",
                "database_id": "retail",
                "dataset_id": "warehouse-shadow",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "ddl_applied_statements": 1,
                "dml_decision": "Applied",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-other",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "schema_version": "schema-v2",
                "release_gate": "post_ddl_dml_release",
                "target_transaction_boundary": "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release",
                "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML may become visible before required acknowledgements reach barrier_lsn",
                "proof_steps": [
                    "target_postgres_recorded_ddl_ack",
                    "release_decision_allowed_post_ddl_dml",
                    "dml_replay_applied_changes_positive",
                    "dml_replay_commit_lsn_matches_ddl_barrier_lsn"
                ]
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "schema_version": "schema-v2",
                "release_gate": "post_ddl_dml_release",
                "target_transaction_boundary": "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release",
                "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until operator release",
                "proof_steps": [
                    "target_postgres_recorded_ddl_ack",
                    "release_decision_allowed_post_ddl_dml",
                    "dml_replay_applied_changes_positive",
                    "dml_replay_commit_lsn_matches_ddl_barrier_lsn"
                ]
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "dml_decision": "SkippedDuplicate",
                "dml_applied_changes": 0,
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
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "0/16B8000",
                "dml_decision": "Applied",
                "dml_applied_changes": 0,
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
            }
        }"#
    ));
}
