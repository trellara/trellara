use serde_json::Value;
use trellara_checkpoint::parse_lsn;

use super::lsn_valid;
use crate::pilot_evidence_identity_helpers::meaningful_json_string;

pub(super) fn verified_apply_identity_present(value: &Value) -> bool {
    string_field_non_empty(value, &["source_id"]) && string_field_non_empty(value, &["dataset_id"])
}

pub(super) fn verified_apply_converged(value: &Value) -> bool {
    bool_field(value, &["converged"]).is_some_and(|converged| converged)
        && verification_watermarks_present(value)
        && table_evidence_non_empty(value)
}

pub(super) fn verified_apply_checksum_matches(value: &Value) -> bool {
    string_field_eq(value, &["checksum_status"], "match")
        && verification_watermarks_present(value)
        && table_evidence_non_empty(value)
}

pub(super) fn verified_apply_target_relation_identity(value: &Value) -> bool {
    json_path(value, &["tables"])
        .and_then(Value::as_array)
        .is_some_and(|tables| !tables.is_empty() && tables.iter().all(relation_identity_declared))
}

pub(super) fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}

fn verification_watermarks_present(value: &Value) -> bool {
    matching_lsn_fields(value, &["source_watermark_lsn"], &["target_watermark_lsn"])
}

fn table_evidence_non_empty(value: &Value) -> bool {
    let Some(table_count) = u64_field(value, &["table_count"]) else {
        return false;
    };
    let Some(tables) = json_path(value, &["tables"]).and_then(Value::as_array) else {
        return false;
    };

    table_count > 0
        && table_count == tables.len() as u64
        && tables.iter().all(table_evidence_matches)
}

fn matching_lsn_fields(value: &Value, left_path: &[&str], right_path: &[&str]) -> bool {
    let Some(left) = json_path(value, left_path).and_then(Value::as_str) else {
        return false;
    };
    let Some(right) = json_path(value, right_path).and_then(Value::as_str) else {
        return false;
    };
    lsn_valid(left) && lsn_valid(right) && parse_lsn(left) == parse_lsn(right)
}

fn table_evidence_matches(value: &Value) -> bool {
    string_field_non_empty(value, &["relation"])
        && bool_field(value, &["converged"]).is_some_and(|converged| converged)
        && string_field_eq(value, &["checksum_status"], "match")
        && relation_identity_matches(value)
}

fn relation_identity_matches(value: &Value) -> bool {
    if bool_field(value, &["relation_match"]).is_some_and(|relation_match| !relation_match) {
        return false;
    }
    let Some(target_relation) = json_path(value, &["target_relation"]).and_then(Value::as_str)
    else {
        return true;
    };
    json_path(value, &["relation"])
        .and_then(Value::as_str)
        .is_some_and(|relation| relation == target_relation)
}

fn relation_identity_declared(value: &Value) -> bool {
    bool_field(value, &["relation_match"]).is_some_and(|relation_match| relation_match)
        && json_path(value, &["target_relation"])
            .and_then(Value::as_str)
            .is_some_and(|target_relation| {
                json_path(value, &["relation"])
                    .and_then(Value::as_str)
                    .is_some_and(|relation| relation == target_relation)
            })
}

fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    json_path(value, path).and_then(Value::as_bool)
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn string_field_non_empty(value: &Value, path: &[&str]) -> bool {
    meaningful_json_string(value, path)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
