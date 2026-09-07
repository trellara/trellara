use serde_json::Value;

use super::json_path;

pub(super) fn propagation_policy_proof(value: &Value) -> bool {
    summary_shape_is_valid(value) || apply_outcome_shape_is_valid(value)
}

pub(super) fn propagation_boundary_proof(value: &Value) -> bool {
    summary_values(value).iter().any(|candidate| {
        string_path(candidate, &["propagation_boundary"])
            .is_some_and(canonical_propagation_boundary)
    })
}

pub(super) fn propagation_decisions_proof(value: &Value) -> bool {
    summary_values(value)
        .iter()
        .any(|candidate| decision_array_is_complete(candidate, &["propagation_decisions"]))
        || decision_string_is_complete(value, &["ddl_propagation_decisions"])
}

pub(super) fn propagation_policy_digest_proof(value: &Value) -> bool {
    summary_values(value)
        .iter()
        .any(|candidate| digest_path_is_valid(candidate, &["propagation_policy_sha256"]))
        || digest_path_is_valid(value, &["ddl_propagation_policy_sha256"])
}

fn summary_values(value: &Value) -> Vec<&Value> {
    let mut values = vec![value];
    if let Some(summary) = json_path(Some(value), &["release_summary"]) {
        values.push(summary);
    }
    values
}

fn summary_shape_is_valid(value: &Value) -> bool {
    summary_values(value).iter().any(|candidate| {
        string_path(candidate, &["propagation_boundary"])
            .is_some_and(canonical_propagation_boundary)
            && decision_array_is_complete(candidate, &["propagation_decisions"])
            && digest_path_is_valid(candidate, &["propagation_policy_sha256"])
    })
}

fn apply_outcome_shape_is_valid(value: &Value) -> bool {
    decision_string_is_complete(value, &["ddl_propagation_decisions"])
        && digest_path_is_valid(value, &["ddl_propagation_policy_sha256"])
}

fn decision_array_is_complete(value: &Value, path: &[&str]) -> bool {
    let Some(items) = json_path(Some(value), path).and_then(Value::as_array) else {
        return false;
    };
    required_decision_prefixes().iter().all(|prefix| {
        items
            .iter()
            .filter_map(Value::as_str)
            .any(|item| count_suffix_is_unsigned(item, prefix))
    })
}

fn decision_string_is_complete(value: &Value, path: &[&str]) -> bool {
    let Some(decisions) = string_path(value, path) else {
        return false;
    };
    if unsigned_path(value, &["ddl_target_ack_required"]).is_none() {
        return false;
    }
    let decisions = decisions
        .strip_prefix("propagation_decisions=")
        .unwrap_or(decisions);
    required_decision_prefixes().iter().all(|prefix| {
        decisions
            .split(',')
            .any(|item| count_suffix_is_unsigned(item.trim(), prefix))
    })
}

fn required_decision_prefixes() -> [&'static str; 4] {
    [
        "auto_apply:",
        "manual_review:",
        "unsupported:",
        "target_ack_required:",
    ]
}

fn count_suffix_is_unsigned(item: &str, prefix: &str) -> bool {
    item.strip_prefix(prefix)
        .is_some_and(|value| value.parse::<usize>().is_ok())
}

fn canonical_propagation_boundary(value: &str) -> bool {
    value == trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY
}

fn digest_path_is_valid(value: &Value, path: &[&str]) -> bool {
    string_path(value, path).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
    })
}

fn unsigned_path(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(Some(value), path).and_then(Value::as_u64)
}

fn string_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    json_path(Some(value), path).and_then(Value::as_str)
}
