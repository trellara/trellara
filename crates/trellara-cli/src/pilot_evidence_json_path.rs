use serde_json::Value;

pub(crate) fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}

pub(crate) fn positive_u64(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_u64)
        .is_some_and(|actual| actual > 0)
}

pub(crate) fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

pub(crate) fn field_contains(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.to_ascii_lowercase().contains(expected))
}

pub(crate) fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
