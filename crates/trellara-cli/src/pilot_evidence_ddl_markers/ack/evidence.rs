use std::collections::BTreeSet;

use serde_json::Value;
use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use super::sinks::required_sink_set;

pub(super) fn item_is_valid(item: &Value, proof: &Value) -> bool {
    let Some(sink) = item.get("sink").and_then(Value::as_str) else {
        return false;
    };
    let Some(ack_lsn) = ack_lsn(item) else {
        return false;
    };

    !sink.trim().is_empty()
        && sink.trim() == sink
        && ack_is_explicitly_accepted(item)
        && lsn_is_valid(ack_lsn)
        && ack_identity_matches(item, proof)
        && ack_matches_barrier_id(item, proof)
        && ack_reaches_barrier(ack_lsn, proof)
        && ack_matches_schema_version(item, proof)
        && super::sink_evidence::is_valid(sink.trim(), item, ack_lsn, proof)
}

pub(super) fn covers_required_sinks(items: &[Value], proof: &Value) -> bool {
    if !items_have_unique_sinks(items) {
        return false;
    }

    let Some(required_sinks) = required_sink_set(proof) else {
        return false;
    };
    if required_sinks.is_empty() {
        return false;
    }

    let acked_sinks: BTreeSet<String> = items
        .iter()
        .filter_map(|item| item.get("sink").and_then(Value::as_str))
        .map(str::trim)
        .filter(|sink| !sink.is_empty())
        .map(ToString::to_string)
        .collect();

    acked_sinks == required_sinks
}

fn items_have_unique_sinks(items: &[Value]) -> bool {
    let mut sinks = BTreeSet::new();
    items
        .iter()
        .filter_map(|item| item.get("sink").and_then(Value::as_str))
        .map(str::trim)
        .all(|sink| !sink.is_empty() && sinks.insert(sink.to_string()))
}

fn ack_lsn(item: &Value) -> Option<&str> {
    item.get("source_ack_lsn")
        .or_else(|| item.get("ack_lsn"))
        .and_then(Value::as_str)
}

fn ack_is_explicitly_accepted(item: &Value) -> bool {
    item.get("accepted").and_then(Value::as_bool) == Some(true)
}

fn ack_matches_barrier_id(item: &Value, proof: &Value) -> bool {
    let Some(expected_barrier_id) = proof.get("barrier_id").and_then(Value::as_str) else {
        return false;
    };
    let Some(ack_barrier_id) = item.get("barrier_id").and_then(Value::as_str) else {
        return false;
    };

    !expected_barrier_id.trim().is_empty() && ack_barrier_id == expected_barrier_id
}

fn ack_identity_matches(item: &Value, proof: &Value) -> bool {
    ["source_id", "database_id", "dataset_id"]
        .iter()
        .all(|field| optional_ack_field_matches(item, proof, field))
}

fn optional_ack_field_matches(item: &Value, proof: &Value, field: &str) -> bool {
    let Some(expected) = proof.get(field).and_then(Value::as_str) else {
        return true;
    };
    let Some(actual) = item.get(field).and_then(Value::as_str) else {
        return false;
    };

    !expected.trim().is_empty() && actual == expected
}

fn ack_reaches_barrier(ack_lsn: &str, proof: &Value) -> bool {
    let Some(barrier_lsn) = proof.get("barrier_lsn").and_then(Value::as_str) else {
        return false;
    };

    lsn_is_valid(barrier_lsn) && parse_lsn(ack_lsn) >= parse_lsn(barrier_lsn)
}

fn ack_matches_schema_version(item: &Value, proof: &Value) -> bool {
    let Some(expected_schema_version) = proof.get("schema_version").and_then(Value::as_str) else {
        return false;
    };
    let Some(ack_schema_version) = item.get("schema_version").and_then(Value::as_str) else {
        return false;
    };

    !expected_schema_version.trim().is_empty() && ack_schema_version == expected_schema_version
}

pub(super) fn lsn_is_valid(lsn: &str) -> bool {
    lsn_shape_is_valid(lsn) && parse_lsn(lsn) > 0
}
