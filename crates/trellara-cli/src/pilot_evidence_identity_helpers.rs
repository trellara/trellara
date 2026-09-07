use serde_json::Value;

#[derive(Clone, Copy)]
pub(crate) struct ExpectedIdentity<'a> {
    pub(crate) source_id: &'a str,
    pub(crate) dataset_id: &'a str,
}

pub(crate) fn source_dataset_identity_present(
    value: &Value,
    source_path: &[&str],
    dataset_path: &[&str],
) -> bool {
    meaningful_json_string(value, source_path) && meaningful_json_string(value, dataset_path)
}

pub(crate) fn source_dataset_identity_matches(
    contents: &str,
    expected: ExpectedIdentity<'_>,
) -> bool {
    json_identity_matches(contents, expected) || text_identity_matches(contents, expected)
}

pub(crate) fn meaningful_json_string(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(meaningful_str)
}

pub(crate) fn meaningful_str(actual: &str) -> bool {
    let trimmed = actual.trim();
    !trimmed.is_empty()
        && !matches!(
            trimmed.to_ascii_lowercase().as_str(),
            "missing" | "none" | "null" | "unknown"
        )
}

fn json_identity_matches(contents: &str, expected: ExpectedIdentity<'_>) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(contents) else {
        return false;
    };
    json_source_matches(&value, expected.source_id)
        && json_dataset_matches(&value, expected.dataset_id)
}

fn json_source_matches(value: &Value, expected: &str) -> bool {
    string_path_eq(value, &["source_id"], expected)
        || string_path_eq(value, &["transaction_boundary", "source_id"], expected)
        || string_path_eq(value, &["release_summary", "source_id"], expected)
        || source_rows_match(value, expected)
        || row_intents_match(value, expected)
}

fn json_dataset_matches(value: &Value, expected: &str) -> bool {
    string_path_eq(value, &["dataset_id"], expected)
        || string_path_eq(value, &["transaction_boundary", "dataset_id"], expected)
        || string_path_eq(value, &["release_summary", "dataset_id"], expected)
}

fn source_rows_match(value: &Value, expected: &str) -> bool {
    json_path(value, &["source_rows"])
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| string_path_eq(row, &["source_id"], expected))
        })
}

fn row_intents_match(value: &Value, expected: &str) -> bool {
    json_path(value, &["row_intents"])
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| string_path_eq(row, &["source_id"], expected))
        })
}

fn text_identity_matches(contents: &str, expected: ExpectedIdentity<'_>) -> bool {
    let lower = contents.to_ascii_lowercase();
    let source_id = expected.source_id.to_ascii_lowercase();
    let dataset_id = expected.dataset_id.to_ascii_lowercase();
    let source_matches = text_value_matches(&lower, "source_id", &source_id)
        || text_value_matches(&lower, "source", &source_id);
    let dataset_matches = text_value_matches(&lower, "dataset_id", &dataset_id)
        || text_value_matches(&lower, "dataset", &dataset_id);
    source_matches && dataset_matches
}

fn string_path_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual == expected)
}

fn text_value_matches(lower: &str, key: &str, expected: &str) -> bool {
    lower
        .split_whitespace()
        .filter_map(|token| token_value(token, key))
        .any(|actual| actual == expected)
        || lower
            .lines()
            .filter_map(|line| line_value(line, key))
            .any(|actual| actual == expected)
}

fn token_value<'a>(token: &'a str, key: &str) -> Option<&'a str> {
    let value = token
        .strip_prefix(&format!("{key}="))
        .or_else(|| token.strip_prefix(&format!("{key}:")))?;
    Some(value.trim_matches(|ch: char| matches!(ch, ',' | ';' | '"' | '\'')))
}

fn line_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let value = line.trim().strip_prefix(&format!("{key}:"))?.trim();
    meaningful_str(value).then_some(value)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
