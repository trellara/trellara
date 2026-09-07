use super::*;

#[test]
fn ddl_release_proof_rejects_mismatched_identity_and_policy_markers() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{"dataset_id":"retail-sales","release_decision":{"release_dml":true,"blocker_codes":[]}}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{"release_summary":{"source_id":"local-source","dataset_id":"unknown"},"release_decision":{"release_dml":true,"blocker_codes":[]}}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "release_summary": {
                "source_id": "local-source",
                "database_id": "retail",
                "dataset_id": "warehouse-shadow"
            },
            "release_decision": {"release_dml": true, "blocker_codes": []}
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "release_summary": {
                "source_id": "local-source",
                "database_id": "analytics",
                "dataset_id": "retail-sales"
            },
            "release_decision": {
                "source_id": "local-source",
                "database_id": "retail",
                "dataset_id": "retail-sales",
                "release_dml": true,
                "blocker_codes": []
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "source_id": "local-source",
            "database_id": "retail",
            "dataset_id": "retail-sales",
            "release_summary": {
                "source_id": "local-source",
                "database_id": "retail",
                "dataset_id": "retail-sales"
            },
            "release_decision": {
                "source_id": "local-source",
                "database_id": "analytics",
                "dataset_id": "retail-sales",
                "release_dml": true,
                "blocker_codes": []
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "release_summary": {
                "source_id": "local-source",
                "dataset_id": "retail-sales",
                "required_sinks": ["raw_cdc_lake", "raw_cdc_lake"]
            },
            "release_decision": {"release_dml": true, "blocker_codes": []}
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "release_summary": {
                "source_id": "local-source",
                "dataset_id": "retail-sales",
                "required_sinks": ["raw_cdc_lake", 42]
            },
            "release_decision": {"release_dml": true, "blocker_codes": []}
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "source dataset identity",
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "required_sinks": ["raw_cdc_lake"],
            "release_summary": {
                "source_id": "local-source",
                "dataset_id": "retail-sales",
                "required_sinks": ["spark_derived_views"]
            },
            "release_decision": {"release_dml": true, "blocker_codes": []}
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_propagation_policy",
        r#"{
            "propagation_boundary": "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks",
            "propagation_decisions": ["auto_apply:1", "manual_review:0", "unsupported:0"],
            "propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }"#
    ));
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_propagation_policy",
        r#"{
            "ddl_propagation_decisions": "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1",
            "ddl_target_ack_required": 1,
            "ddl_propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }"#
    ));
    let nested_release_summary = r#"{
        "release_summary": {
            "propagation_boundary": "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks",
            "propagation_decisions": ["auto_apply:1", "manual_review:0", "unsupported:0", "target_ack_required:1"],
            "propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }
    }"#;
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "propagation_boundary",
        nested_release_summary
    ));
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "propagation_decisions",
        nested_release_summary
    ));
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "propagation_policy_sha256",
        nested_release_summary
    ));
}
