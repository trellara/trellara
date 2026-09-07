use serde_json::Value;

use super::lsn_present;
use crate::pilot_evidence_identity_helpers::source_dataset_identity_present as json_source_dataset_identity_present;

pub(super) fn source_dataset_identity_present(value: &Value) -> bool {
    json_source_dataset_identity_present(value, &["source_id"], &["dataset_id"])
}

pub(super) fn snapshot_handoff_ready(value: &Value) -> bool {
    string_field(value, &["state"]).is_some_and(snapshot_handoff_state)
        && source_dataset_identity_present(value)
        && string_field_non_empty(value, &["run_id"])
        && string_field_non_empty(value, &["slot"])
        && selected_table_coverage(value)
        && snapshot_tables_complete(value)
        && watermark_consistent(value)
}

pub(super) fn selected_table_coverage(value: &Value) -> bool {
    let Some(selected_table_count) = u64_field(value, &["selected_table_count"]) else {
        return false;
    };
    let Some(table_count) = u64_field(value, &["table_count"]) else {
        return false;
    };
    let Some(tables) = tables(value) else {
        return false;
    };

    selected_table_count > 0
        && selected_table_count == table_count
        && selected_table_count == tables.len() as u64
}

pub(super) fn snapshot_tables_complete(value: &Value) -> bool {
    let Some(tables) = tables(value) else {
        return false;
    };
    let Some(table_count) = u64_field(value, &["table_count"]) else {
        return false;
    };

    !tables.is_empty()
        && table_count == tables.len() as u64
        && tables.iter().all(|table| {
            string_field_non_empty(table, &["relation"])
                && string_field_eq(table, &["state"], "copy_complete")
                && lsn_valid_field(table, &["watermark_lsn"])
        })
}

pub(super) fn watermark_consistent(value: &Value) -> bool {
    let Some(consistent_lsn) = string_field(value, &["consistent_lsn"]) else {
        return false;
    };
    if !lsn_present(consistent_lsn) {
        return false;
    }

    tables(value).is_some_and(|tables| {
        !tables.is_empty()
            && tables.iter().all(|table| {
                string_field(table, &["watermark_lsn"])
                    .is_some_and(|watermark| watermark == consistent_lsn)
            })
    })
}

pub(super) fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}

fn snapshot_handoff_state(state: &str) -> bool {
    matches!(
        state.to_ascii_lowercase().as_str(),
        "stream_handoff_ready" | "streaming" | "verified"
    )
}

fn tables(value: &Value) -> Option<&Vec<Value>> {
    json_path(value, &["tables"]).and_then(Value::as_array)
}

fn lsn_valid_field(value: &Value, path: &[&str]) -> bool {
    string_field(value, path).is_some_and(super::lsn_valid)
}

fn string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    json_path(value, path).and_then(Value::as_str)
}

fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    string_field(value, path).is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn string_field_non_empty(value: &Value, path: &[&str]) -> bool {
    string_field(value, path).is_some_and(|actual| !actual.trim().is_empty())
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
