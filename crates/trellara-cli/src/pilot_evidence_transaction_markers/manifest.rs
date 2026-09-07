use serde_json::Value;

use super::manifest_partitions::{
    json_participating_partition_count_matches, json_partition_ids_present,
    text_participating_partition_count_matches, text_partition_ids_present,
};

const PARTITIONED_SCALE_VISIBILITY_CONTRACT: &str =
    "global visibility waits for manifest, commit marker, and every participating partition";

pub(super) fn manifest_evidence_headers(contents: &str, lower: &str) -> bool {
    if lower.contains("parallel replay is disabled") {
        return true;
    }
    text_manifest_evidence_headers(lower) || json_manifest_evidence_headers(contents)
}

fn text_manifest_evidence_headers(lower: &str) -> bool {
    positive_text_usize(lower, "partitioned_scale_manifest_checksum")
        && positive_text_usize(lower, "partitioned_scale_manifest_event_count")
        && positive_text_usize(lower, "partitioned_scale_envelope_event_count")
        && text_manifest_counts_match(lower)
        && lower.contains("partitioned_scale_event_count_coverage=true")
        && text_partition_ids_present(lower)
        && text_participating_partition_count_matches(lower)
        && text_visibility_contract_matches(lower)
}

fn json_manifest_evidence_headers(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    positive_usize_at_either(&value, "partitioned_scale_manifest_checksum")
        && positive_usize_at_either(&value, "partitioned_scale_manifest_event_count")
        && positive_usize_at_either(&value, "partitioned_scale_envelope_event_count")
        && json_manifest_counts_match(&value)
        && bool_at_either(&value, "partitioned_scale_event_count_coverage") == Some(true)
        && json_partition_ids_present(&value)
        && json_participating_partition_count_matches(&value)
        && string_at_either(&value, "partitioned_scale_visibility_contract").is_some_and(
            |contract| contract.eq_ignore_ascii_case(PARTITIONED_SCALE_VISIBILITY_CONTRACT),
        )
}

fn positive_text_usize(lower: &str, key: &str) -> bool {
    text_usize(lower, key).is_some_and(|count| count > 0)
}

fn text_manifest_counts_match(lower: &str) -> bool {
    match (
        text_usize(lower, "partitioned_scale_manifest_event_count"),
        text_usize(lower, "partitioned_scale_envelope_event_count"),
    ) {
        (Some(manifest), Some(envelope)) => manifest == envelope,
        _ => false,
    }
}

fn text_usize(lower: &str, key: &str) -> Option<usize> {
    lower.lines().find_map(|line| {
        value_for_key_in_line(line, key).and_then(|value| value.parse::<usize>().ok())
    })
}

fn text_visibility_contract_matches(lower: &str) -> bool {
    lower.lines().any(|line| {
        line_value_for_key(line, "partitioned_scale_visibility_contract")
            .is_some_and(|contract| contract == PARTITIONED_SCALE_VISIBILITY_CONTRACT)
    })
}

fn positive_usize_at_either(value: &Value, key: &str) -> bool {
    json_usize_at_either(value, key).is_some_and(|count| count > 0)
}

fn json_manifest_counts_match(value: &Value) -> bool {
    match (
        json_usize_at_either(value, "partitioned_scale_manifest_event_count"),
        json_usize_at_either(value, "partitioned_scale_envelope_event_count"),
    ) {
        (Some(manifest), Some(envelope)) => manifest == envelope,
        _ => false,
    }
}

fn json_usize_at_either(value: &Value, key: &str) -> Option<u64> {
    json_path_either(value, key).and_then(Value::as_u64)
}

fn bool_at_either(value: &Value, key: &str) -> Option<bool> {
    json_path_either(value, key).and_then(Value::as_bool)
}

fn string_at_either<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    json_path_either(value, key).and_then(Value::as_str)
}

fn json_path_either<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    json_path(Some(value), &[key])
        .or_else(|| json_path(Some(value), &["transaction_boundary", key]))
}

fn value_for_key_in_line<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace()
        .find_map(|token| token_value_for_key(token, key))
}

fn line_value_for_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let value = line
        .strip_prefix(&format!("{key}="))
        .or_else(|| line.strip_prefix(&format!("{key}:")))?;
    Some(value.trim_matches(|ch: char| matches!(ch, ',' | ';' | '"' | '\'' | '{' | '}')))
        .filter(|value| !value.is_empty())
}

fn token_value_for_key<'a>(token: &'a str, key: &str) -> Option<&'a str> {
    let value = token
        .strip_prefix(&format!("{key}="))
        .or_else(|| token.strip_prefix(&format!("{key}:")))?;
    Some(value.trim_matches(|ch: char| matches!(ch, ',' | ';' | '"' | '\'' | '{' | '}')))
        .filter(|value| !value.is_empty())
}

fn json_path<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value?, |current, key| current.get(*key))
}

fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}
