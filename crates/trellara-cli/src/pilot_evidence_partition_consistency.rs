use serde_json::Value;

use super::pilot_evidence_partition_json::json_path;

pub(crate) fn string_fields_match(value: &Value, left_path: &[&str], right_path: &[&str]) -> bool {
    let Some(left) = json_path(value, left_path).and_then(Value::as_str) else {
        return false;
    };
    let Some(right) = json_path(value, right_path).and_then(Value::as_str) else {
        return false;
    };
    !left.trim().is_empty() && left == right
}

pub(crate) fn u64_fields_match(value: &Value, left_path: &[&str], right_path: &[&str]) -> bool {
    let Some(left) = json_path(value, left_path).and_then(Value::as_u64) else {
        return false;
    };
    json_path(value, right_path)
        .and_then(Value::as_u64)
        .is_some_and(|right| right == left)
}
