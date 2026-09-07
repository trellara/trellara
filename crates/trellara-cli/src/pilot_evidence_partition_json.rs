use serde_json::Value;
use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

pub(super) fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    json_path(value, path).and_then(Value::as_bool)
}

pub(super) fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

pub(super) fn string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    json_path(value, path).and_then(Value::as_str)
}

pub(super) fn array_field_empty(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
}

pub(super) fn lsn_field_valid(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|lsn| lsn_shape_is_valid(lsn) && parse_lsn(lsn) > 0)
}

pub(super) fn lsn_field_at_or_after(value: &Value, high_path: &[&str], low_path: &[&str]) -> bool {
    let Some(high) = json_path(value, high_path).and_then(Value::as_str) else {
        return false;
    };
    let Some(low) = json_path(value, low_path).and_then(Value::as_str) else {
        return false;
    };
    parse_lsn(high) >= parse_lsn(low)
}

pub(super) fn lsn_fields_match(value: &Value, left_path: &[&str], right_path: &[&str]) -> bool {
    let Some(left) = json_path(value, left_path).and_then(Value::as_str) else {
        return false;
    };
    let Some(right) = json_path(value, right_path).and_then(Value::as_str) else {
        return false;
    };
    lsn_shape_is_valid(left) && lsn_shape_is_valid(right) && parse_lsn(left) == parse_lsn(right)
}

pub(crate) fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
