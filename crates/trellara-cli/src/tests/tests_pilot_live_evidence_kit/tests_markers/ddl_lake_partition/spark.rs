use super::*;

const TEMPLATE_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn spark_derived_views_ack_evidence_requires_template_digest_detail() {
    let proof = format!(
        r#"{{
            "release_decision": {{"release_dml": true}},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["spark_derived_views"],
            "ack_evidence": [
                {{"sink": "spark_derived_views", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "Spark-derived views accepted 2 regenerated templates; template_digest={TEMPLATE_DIGEST}; accepted_by=platform-review; release_gate=post_ddl_dml_release"}}
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
            "required_sinks": ["spark_derived_views"],
            "ack_evidence": [
                {"sink": "spark_derived_views", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ]
        }"#
    ));

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["spark_derived_views"],
            "ack_evidence": [
                {"sink": "spark_derived_views", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "Spark-derived views accepted 2 regenerated templates; template_digest=sha256:templates-v2; accepted_by=platform-review; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));

    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_evidence",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["spark_derived_views"],
            "ack_evidence": [
                {"sink": "spark_derived_views", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "Spark-derived views accepted 2 stale templates; template_digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef; accepted_by=platform-review; release_gate=post_ddl_dml_release"}
            ]
        }"#
    ));
}

#[test]
fn spark_derived_views_ack_commands_require_template_digest_flags() {
    let command = format!(
        "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink spark_derived_views --ack-lsn 0/16B9000 --schema-version schema-v2 --template-digest {TEMPLATE_DIGEST} --accepted-by platform-review --view-count 2"
    );
    let proof = format!(
        r#"{{
            "release_decision": {{"release_dml": true}},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["spark_derived_views"],
            "ack_commands": ["{command}"]
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
            "required_sinks": ["spark_derived_views"],
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink spark_derived_views --ack-lsn 0/16B9000 --schema-version schema-v2 --template-digest sha256:templates-v2 --accepted-by platform-review --view-count 2"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "release_decision": {"release_dml": true},
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["spark_derived_views"],
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink spark_derived_views --ack-lsn 0/16B9000 --schema-version schema-v2 --template-digest 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef --accepted-by --view-count 2"
            ]
        }"#
    ));
}
