use serde_json::Value;
use trellara_protocol::parse_lsn;

use super::json_path;

const DDL_DML_REPLAY_PROOF_CONTRACT: &str =
    "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary";
const TARGET_DDL_TRANSACTION_BOUNDARY: &str =
    "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release";

pub(super) fn ddl_dml_replay_proof(value: &Value) -> bool {
    let proof = json_path(Some(value), &["ddl_dml_replay_proof"]).unwrap_or(value);
    string_field(proof, "contract")
        .is_some_and(|contract| contract == DDL_DML_REPLAY_PROOF_CONTRACT)
        && release_gate_is_post_ddl(proof)
        && target_boundary_matches(proof)
        && cdc_boundary_holds_post_ddl_dml(proof)
        && ddl_replay_applied_schema_changes(proof)
        && dml_replay_applied_changes(proof)
        && lsn_field_matches(proof, "target_ack_lsn", "barrier_lsn")
        && lsn_field_matches(proof, "dml_commit_lsn", "barrier_lsn")
        && release_decision_allows_post_ddl_dml(value)
        && same_optional_field(proof, value, "source_id")
        && same_optional_field(proof, value, "database_id")
        && same_optional_field(proof, value, "dataset_id")
        && same_optional_field(proof, value, "barrier_id")
        && same_optional_field(proof, value, "schema_version")
        && proof_steps_complete(proof)
}

fn release_decision_allows_post_ddl_dml(value: &Value) -> bool {
    json_path(Some(value), &["release_decision", "release_dml"]).and_then(Value::as_bool)
        == Some(true)
        && json_path(Some(value), &["release_decision", "blocker_codes"])
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
}

fn release_gate_is_post_ddl(proof: &Value) -> bool {
    string_field(proof, "release_gate").is_some_and(|gate| gate == "post_ddl_dml_release")
}

fn target_boundary_matches(proof: &Value) -> bool {
    string_field(proof, "target_transaction_boundary")
        .is_some_and(|boundary| boundary == TARGET_DDL_TRANSACTION_BOUNDARY)
}

fn cdc_boundary_holds_post_ddl_dml(proof: &Value) -> bool {
    string_field(proof, "cdc_transaction_boundary").is_some_and(|boundary| {
        boundary.contains("source commit LSN is the DDL barrier")
            && boundary.contains("post-DDL DML stays invisible")
            && (boundary.contains("required sink ACK")
                || boundary.contains("required acknowledgements"))
            && boundary.contains("barrier_lsn")
    })
}

fn dml_replay_applied_changes(proof: &Value) -> bool {
    string_field(proof, "dml_decision").is_some_and(|decision| decision == "Applied")
        && json_path(Some(proof), &["dml_applied_changes"])
            .and_then(Value::as_u64)
            .is_some_and(|applied_changes| applied_changes > 0)
}

fn ddl_replay_applied_schema_changes(proof: &Value) -> bool {
    json_path(Some(proof), &["ddl_applied_statements"])
        .and_then(Value::as_u64)
        .is_some_and(|applied_statements| applied_statements > 0)
}

fn lsn_field_matches(proof: &Value, left: &'static str, right: &'static str) -> bool {
    let Some(left) = string_field(proof, left) else {
        return false;
    };
    let Some(right) = string_field(proof, right) else {
        return false;
    };
    parse_lsn(left).ok() == parse_lsn(right).ok()
}

fn same_optional_field(proof: &Value, value: &Value, field: &'static str) -> bool {
    let Some(top_level) = string_field(value, field) else {
        return true;
    };
    string_field(proof, field).is_some_and(|proof_value| proof_value == top_level)
}

fn proof_steps_complete(proof: &Value) -> bool {
    let Some(steps) = json_path(Some(proof), &["proof_steps"]).and_then(Value::as_array) else {
        return false;
    };
    [
        "target_postgres_recorded_ddl_ack",
        "release_decision_allowed_post_ddl_dml",
        "dml_replay_applied_changes_positive",
        "dml_replay_commit_lsn_matches_ddl_barrier_lsn",
    ]
    .iter()
    .all(|required| {
        steps
            .iter()
            .filter_map(Value::as_str)
            .any(|step| step == *required)
    })
}

fn string_field<'a>(value: &'a Value, field: &'static str) -> Option<&'a str> {
    json_path(Some(value), &[field]).and_then(Value::as_str)
}
