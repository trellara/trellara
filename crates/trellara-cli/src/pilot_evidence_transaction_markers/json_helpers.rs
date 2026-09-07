use serde_json::Value;

pub(super) fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}

pub(super) fn json_path<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value?, |current, key| current.get(*key))
}

pub(super) fn string_at<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a str> {
    json_path(value, path).and_then(Value::as_str)
}

pub(super) fn bool_at(value: Option<&Value>, path: &[&str]) -> Option<bool> {
    json_path(value, path).and_then(Value::as_bool)
}

pub(super) fn u64_at(value: Option<&Value>, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

pub(super) fn string_at_either<'a>(
    value: &'a Value,
    top_level_key: &'static str,
) -> Option<&'a str> {
    string_at(Some(value), &[top_level_key])
        .or_else(|| string_at(Some(value), &["transaction_boundary", top_level_key]))
}

pub(super) fn bool_at_either(value: &Value, top_level_key: &'static str) -> Option<bool> {
    bool_at(Some(value), &[top_level_key])
        .or_else(|| bool_at(Some(value), &["transaction_boundary", top_level_key]))
}

pub(super) fn u64_at_either(value: &Value, top_level_key: &'static str) -> Option<u64> {
    u64_at(Some(value), &[top_level_key])
        .or_else(|| u64_at(Some(value), &["transaction_boundary", top_level_key]))
}
