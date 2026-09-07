use std::collections::BTreeSet;

use serde_json::Value;

pub(super) fn text_partition_ids_present(lower: &str) -> bool {
    lower.lines().any(|line| {
        value_for_key_in_line(line, "partitioned_scale_participating_partition_ids")
            .is_some_and(text_partition_ids_valid)
    })
}

pub(super) fn text_participating_partition_count_matches(lower: &str) -> bool {
    let Some(expected_count) = text_partition_count(lower) else {
        return false;
    };
    lower.lines().any(|line| {
        value_for_key_in_line(line, "partitioned_scale_participating_partition_ids")
            .and_then(partition_id_count)
            .is_some_and(|actual_count| actual_count == expected_count)
    })
}

pub(super) fn json_partition_ids_present(value: &Value) -> bool {
    json_path_either(value, "partitioned_scale_participating_partition_ids").is_some_and(
        |partition_ids| {
            partition_ids
                .as_array()
                .is_some_and(|ids| json_partition_id_array_valid(ids))
                || partition_ids.as_str().is_some_and(text_partition_ids_valid)
        },
    )
}

pub(super) fn json_participating_partition_count_matches(value: &Value) -> bool {
    let Some(expected_count) = json_partition_count(value) else {
        return false;
    };
    json_path_either(value, "partitioned_scale_participating_partition_ids").is_some_and(
        |partition_ids| {
            partition_ids
                .as_array()
                .and_then(|ids| json_partition_id_count(ids))
                .or_else(|| {
                    partition_ids
                        .as_str()
                        .and_then(partition_id_count)
                        .and_then(|count| u64::try_from(count).ok())
                })
                .is_some_and(|actual_count| actual_count == expected_count)
        },
    )
}

fn text_partition_count(lower: &str) -> Option<usize> {
    text_usize(lower, "partitioned_scale_participating_partition_count")
        .or_else(|| text_usize(lower, "participating_partition_count"))
}

fn text_partition_ids_valid(ids: &str) -> bool {
    partition_id_count(ids).is_some()
}

fn partition_id_count(ids: &str) -> Option<usize> {
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    for id in ids.split(',').map(str::trim) {
        let Ok(id) = id.parse::<u32>() else {
            return None;
        };
        if !seen.insert(id) {
            return None;
        }
        count += 1;
    }
    (count > 0).then_some(count)
}

fn json_partition_count(value: &Value) -> Option<u64> {
    json_usize_at_either(value, "partitioned_scale_participating_partition_count")
        .or_else(|| json_usize_at_either(value, "participating_partition_count"))
}

fn json_partition_id_array_valid(ids: &[Value]) -> bool {
    json_partition_id_count(ids).is_some()
}

fn json_partition_id_count(ids: &[Value]) -> Option<u64> {
    let mut seen = BTreeSet::new();
    let count = u64::try_from(ids.len()).ok()?;
    (!ids.is_empty()
        && ids.iter().all(|id| {
            id.as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .is_some_and(|id| seen.insert(id))
        }))
    .then_some(count)
}

fn text_usize(lower: &str, key: &str) -> Option<usize> {
    lower.lines().find_map(|line| {
        value_for_key_in_line(line, key).and_then(|value| value.parse::<usize>().ok())
    })
}

fn json_usize_at_either(value: &Value, key: &str) -> Option<u64> {
    json_path_either(value, key).and_then(Value::as_u64)
}

fn json_path_either<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    json_path(Some(value), &[key])
        .or_else(|| json_path(Some(value), &["transaction_boundary", key]))
}

fn value_for_key_in_line<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace()
        .find_map(|token| token_value_for_key(token, key))
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
