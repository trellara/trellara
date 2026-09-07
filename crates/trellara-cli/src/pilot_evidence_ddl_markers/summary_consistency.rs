use std::collections::BTreeSet;

use serde_json::Value;

use super::json_path;

pub(super) fn release_summary_matches_top_level(value: &Value) -> bool {
    scalar_fields_match(value)
        && release_decision_boundary_matches(value)
        && release_decision_matches(value)
        && blocker_codes_match(value)
        && required_sinks_match(value)
}

fn scalar_fields_match(value: &Value) -> bool {
    [
        "source_id",
        "database_id",
        "dataset_id",
        "barrier_id",
        "barrier_lsn",
        "schema_version",
    ]
    .iter()
    .all(|field| optional_values_match(value, &[*field], &["release_summary", *field]))
}

fn release_decision_matches(value: &Value) -> bool {
    optional_values_match(
        value,
        &["release_decision", "release_dml"],
        &["release_summary", "release_dml"],
    )
}

fn release_decision_boundary_matches(value: &Value) -> bool {
    [
        "source_id",
        "database_id",
        "dataset_id",
        "barrier_id",
        "barrier_lsn",
        "cdc_transaction_boundary",
    ]
    .iter()
    .all(|field| release_decision_field_matches(value, field))
}

fn release_decision_field_matches(value: &Value, field: &str) -> bool {
    let decision_path = &["release_decision", field];
    optional_values_match(value, decision_path, &["release_summary", field])
        && optional_values_match(value, decision_path, &[field])
}

fn blocker_codes_match(value: &Value) -> bool {
    optional_values_match(
        value,
        &["release_decision", "blocker_codes"],
        &["release_summary", "blocker_codes"],
    )
}

fn required_sinks_match(value: &Value) -> bool {
    match (
        strict_string_set_at(value, &["required_sinks"]),
        strict_string_set_at(value, &["release_summary", "required_sinks"]),
    ) {
        (SinkSet::Present(top_level), SinkSet::Present(summary)) => top_level == summary,
        (SinkSet::Invalid, _) | (_, SinkSet::Invalid) => false,
        _ => true,
    }
}

fn optional_values_match(value: &Value, top_level_path: &[&str], summary_path: &[&str]) -> bool {
    let top_level = json_path(Some(value), top_level_path);
    let summary = json_path(Some(value), summary_path);
    match (top_level, summary) {
        (Some(top_level), Some(summary)) => top_level == summary,
        _ => true,
    }
}

enum SinkSet {
    Missing,
    Invalid,
    Present(BTreeSet<String>),
}

fn strict_string_set_at(value: &Value, path: &[&str]) -> SinkSet {
    let Some(items) = json_path(Some(value), path) else {
        return SinkSet::Missing;
    };
    let Some(items) = items.as_array() else {
        return SinkSet::Invalid;
    };

    let mut sinks = BTreeSet::new();
    for item in items {
        let Some(sink) = item.as_str().map(str::trim) else {
            return SinkSet::Invalid;
        };
        if sink.is_empty() || !sinks.insert(sink.to_string()) {
            return SinkSet::Invalid;
        }
    }
    SinkSet::Present(sinks)
}
