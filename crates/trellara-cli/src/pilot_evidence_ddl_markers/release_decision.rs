use serde_json::Value;

use super::{ack, blockers, cdc_boundary, json_path, release_gate};

pub(super) fn release_dml_true(value: &Value) -> bool {
    decision_releases_dml(value)
        && blockers::has_no_release_blockers(value)
        && cdc_boundary_is_explicit(value)
        && release_gate::post_ddl_release_gate_satisfied(value)
        && ack_commands_are_complete(value)
        && ack_evidence_is_complete(value)
}

fn decision_releases_dml(value: &Value) -> bool {
    value
        .get("release_decision")
        .and_then(|decision| decision.get("release_dml"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn cdc_boundary_is_explicit(value: &Value) -> bool {
    cdc_boundary::transaction_boundary_present(value)
}

fn ack_commands_are_complete(value: &Value) -> bool {
    let Some(items) = json_path(Some(value), &["ack_commands"]).and_then(Value::as_array) else {
        return false;
    };

    !items.is_empty()
        && items.iter().all(ack::command_is_executable)
        && ack::command_collection_is_valid(items, value)
}

fn ack_evidence_is_complete(value: &Value) -> bool {
    let Some(items) = json_path(Some(value), &["ack_evidence"]).and_then(Value::as_array) else {
        return false;
    };

    !items.is_empty()
        && items
            .iter()
            .all(|item| ack::evidence_item_is_valid(item, value))
        && ack::evidence_covers_required_sinks(items, value)
}
