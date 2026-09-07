use super::*;
use serde_json::Value;

#[test]
fn ddl_release_proof_rejects_release_decision_barrier_mismatch() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        &proof_with_decision_field("barrier_lsn", r#""0/16B9000""#)
    ));
}

#[test]
fn ddl_release_proof_rejects_release_decision_cdc_boundary_mismatch() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        &proof_with_decision_field(
            "cdc_transaction_boundary",
            r#""target timestamp released post-DDL DML""#
        )
    ));
}

fn proof_with_decision_field(field: &str, value: &str) -> String {
    let mut proof: Value = serde_json::from_str(&valid_release_proof()).expect("valid proof json");
    proof["release_decision"][field] = serde_json::from_str(value).expect("field value json");
    serde_json::to_string(&proof).expect("proof json")
}

fn valid_release_proof() -> String {
    r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "barrier_id": "ddl-barrier-abc",
        "barrier_lsn": "0/16B8000",
        "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
        "release_summary": {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "release_dml": true,
            "blocker_codes": [],
            "required_sinks": ["raw_cdc_lake"]
        },
        "release_decision": {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "release_dml": true,
            "blocker_codes": []
        },
        "required_sinks": ["raw_cdc_lake"]
    }"#
    .to_string()
}
