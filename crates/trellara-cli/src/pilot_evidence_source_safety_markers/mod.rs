use serde_json::Value;

use crate::pilot_evidence_identity_helpers::{meaningful_json_string, meaningful_str};

mod json;
mod text;

pub(crate) fn source_safety_marker_present(marker: &str, contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    let value = json_value(contents);

    match marker {
        "source dataset identity" => {
            value.as_ref().is_some_and(json::identity_present) || text::identity_present(&lower)
        }
        "Trellara source safety" => {
            lower.contains("trellara source safety")
                || value.as_ref().is_some_and(json::has_source_safety_shape)
        }
        "no critical findings" => {
            value.as_ref().is_some_and(json::has_no_critical_findings)
                || text::has_no_critical_findings(&lower)
        }
        "ready_or_warning_status" => {
            value.as_ref().is_some_and(json::status_is_acceptable)
                || text::status_is_acceptable(&lower)
        }
        _ => false,
    }
}

fn acceptable_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "ready" | "warning" | "healthy" | "degraded"
    )
}

fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn string_field_non_empty(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| !actual.trim().is_empty())
}

fn meaningful_string_field(value: &Value, path: &[&str]) -> bool {
    meaningful_json_string(value, path)
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}

fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}
