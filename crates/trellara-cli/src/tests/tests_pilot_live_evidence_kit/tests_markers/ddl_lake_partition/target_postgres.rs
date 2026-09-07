use super::*;

#[test]
fn target_postgres_ack_evidence_requires_digest_detail() {
    let valid_detail = format!(
        "target Postgres applied 1 DDL statements; plan_sha256={}; statement_sha256={}; release_gate=post_ddl_dml_release",
        "a".repeat(64),
        "b".repeat(64)
    );
    let proof = format!(
        r#"{{
            "release_decision": {{"release_dml": true}},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_evidence": [
                {{"sink": "target_postgres", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "{valid_detail}"}}
            ]
        }}"#
    );

    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        &proof
    ));

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_evidence": [
                {"sink": "target_postgres", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));
}

#[test]
fn target_postgres_ack_evidence_rejects_mismatched_barrier_lsn_detail() {
    let detail = format!(
        "target Postgres applied 1 DDL statements; plan_sha256={}; statement_sha256={}; barrier_lsn=0/16C8000; release_gate=post_ddl_dml_release",
        "a".repeat(64),
        "b".repeat(64)
    );
    let proof = format!(
        r#"{{
            "release_decision": {{"release_dml": true}},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_evidence": [
                {{"sink": "target_postgres", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16C8000", "schema_version": "schema-v2", "accepted": true, "detail": "{detail}"}}
            ]
        }}"#
    );

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        &proof
    ));
}

#[test]
fn target_postgres_ack_commands_require_digest_flags() {
    let valid_command = format!(
        "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink target_postgres --ack-lsn 0/16B9000 --schema-version schema-v2 --plan-sha256 {} --statement-sha256 {}",
        "a".repeat(64),
        "b".repeat(64)
    );
    let proof = format!(
        r#"{{
            "release_decision": {{"release_dml": true}},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_commands": ["{valid_command}"]
        }}"#
    );

    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        &proof
    ));

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink target_postgres --ack-lsn 0/16B9000 --schema-version schema-v2"
            ]
        }"#
    ));

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres"],
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink target_postgres --ack-lsn 0/16B9000 --schema-version schema-v2 --plan-sha256 short --statement-sha256 bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            ]
        }"#
    ));
}
