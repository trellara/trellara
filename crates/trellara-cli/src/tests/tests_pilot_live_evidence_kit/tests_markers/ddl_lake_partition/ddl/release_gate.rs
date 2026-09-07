use super::*;

#[test]
fn ddl_release_proof_rejects_unsatisfied_release_gate() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": false}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        "operator note: post_ddl_dml_release"
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_gate": "post_ddl_dml_release",
            "release_decision": {"release_dml": true, "blocker_codes": []}
        }"#
    ));
    assert!(live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_summary": {
                "release_gates": [
                    {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
                ]
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_summary": {
                "barrier_id": "ddl-barrier-old",
                "release_gates": [
                    {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
                ]
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_gate": "post_ddl_dml_release",
            "release_decision": {"release_dml": false, "blocker_codes": ["pending_required_sink_ack"]}
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_decision": {"release_dml": false, "blocker_codes": []},
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_decision": {"blocker_codes": []},
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "no release blockers",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": ["pending_required_sink_ack"]},
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": ["pending_required_sink_ack"]},
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{
            "release_summary": {"release_dml": true, "blocker_codes": []},
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{
            "release_decision": {"release_dml": false, "blocker_codes": []},
            "release_summary": {"release_dml": true, "blocker_codes": []},
            "release_blocker_codes": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "post_ddl_dml_release",
        r#"{
            "release_decision": {"release_dml": true, "blocker_codes": []},
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ],
            "release_blocker_codes": ["pending_required_sink_ack"]
        }"#
    ));
}
