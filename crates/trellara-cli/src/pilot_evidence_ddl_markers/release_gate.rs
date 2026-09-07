use serde_json::Value;

use super::json_path;

pub(super) fn post_ddl_release_gate_satisfied(value: &Value) -> bool {
    release_decision_is_structurally_released(value)
        && (release_gate_collection_satisfies(json_path(Some(value), &["release_gates"]))
            || release_gate_collection_satisfies(json_path(
                Some(value),
                &["release_summary", "release_gates"],
            )))
}

fn release_gate_collection_satisfies(value: Option<&Value>) -> bool {
    let Some(items) = value.and_then(Value::as_array) else {
        return false;
    };

    items.iter().any(post_ddl_release_gate_is_satisfied)
}

fn post_ddl_release_gate_is_satisfied(item: &Value) -> bool {
    let gate_code = item
        .get("release_gate_code")
        .or_else(|| item.get("name"))
        .and_then(Value::as_str);
    gate_code.is_some_and(|gate| gate == "post_ddl_dml_release")
        && item
            .get("satisfied")
            .and_then(Value::as_bool)
            .is_some_and(|satisfied| satisfied)
}

fn release_decision_is_structurally_released(value: &Value) -> bool {
    value
        .get("release_decision")
        .and_then(|decision| decision.get("release_dml"))
        .and_then(Value::as_bool)
        == Some(true)
}
