use serde_json::Value;
use trellara_protocol::PARTITION_REBALANCE_VISIBILITY_CONTRACT;

use crate::pilot_evidence_identity_helpers::source_dataset_identity_present as json_source_dataset_identity_present;

pub(crate) fn partition_rebalance_marker_present(marker: &str, contents: &str) -> bool {
    let Some(value) = serde_json::from_str::<Value>(contents).ok() else {
        return false;
    };

    match marker {
        "source dataset identity" => source_dataset_identity_present(&value),
        "rebalance evidence complete" => rebalance_evidence_complete(&value),
        "runtime movement disabled" => runtime_movement_disabled(&value),
        "visibility contract present" => visibility_contract_present(&value),
        "skew metrics present" => skew_metrics_present(&value),
        "move candidates reviewable" => move_candidates_reviewable(&value),
        _ => false,
    }
}

fn source_dataset_identity_present(value: &Value) -> bool {
    json_source_dataset_identity_present(value, &["source_id"], &["dataset_id"])
}

fn rebalance_evidence_complete(value: &Value) -> bool {
    bool_field(value, &["evidence_complete"]).is_some_and(|complete| complete)
        && string_field(value, &["status"])
            .is_some_and(|status| matches!(status, "Stable" | "Skewed" | "stable" | "skewed"))
        && array_field_empty(value, &["missing_partitions"])
        && array_field_empty(value, &["blocking_partition_ids"])
        && positive_u64_field(value, &["expected_partition_count"])
}

fn runtime_movement_disabled(value: &Value) -> bool {
    bool_field(value, &["runtime_movement_allowed"]).is_some_and(|allowed| !allowed)
}

fn visibility_contract_present(value: &Value) -> bool {
    string_field(value, &["visibility_contract"])
        .is_some_and(|contract| contract == PARTITION_REBALANCE_VISIBILITY_CONTRACT)
}

fn skew_metrics_present(value: &Value) -> bool {
    let Some(min_event_count) = u64_field(value, &["min_event_count"]) else {
        return false;
    };
    let Some(max_event_count) = u64_field(value, &["max_event_count"]) else {
        return false;
    };
    let Some(total_event_count) = u64_field(value, &["total_event_count"]) else {
        return false;
    };

    u64_field(value, &["max_skew_percent"]).is_some()
        && min_event_count <= max_event_count
        && total_event_count >= max_event_count
        && skew_ratio_present(value, min_event_count, max_event_count, total_event_count)
}

fn skew_ratio_present(
    value: &Value,
    min_event_count: u64,
    max_event_count: u64,
    total_event_count: u64,
) -> bool {
    if max_event_count == 0 {
        return min_event_count == 0
            && total_event_count == 0
            && json_path(value, &["skew_ratio_basis_points"]).is_some_and(Value::is_null);
    }
    positive_u64_field(value, &["skew_ratio_basis_points"])
}

fn move_candidates_reviewable(value: &Value) -> bool {
    let Some(moves) = value.get("recommended_moves").and_then(Value::as_array) else {
        return false;
    };
    let Some(expected_partition_count) = u64_field(value, &["expected_partition_count"]) else {
        return false;
    };
    if status_is_skewed(value) && moves.is_empty() {
        return false;
    }

    moves
        .iter()
        .all(|candidate| move_candidate_reviewable(candidate, expected_partition_count))
}

fn move_candidate_reviewable(candidate: &Value, expected_partition_count: u64) -> bool {
    let Some(from_partition_id) = u64_field(candidate, &["from_partition_id"]) else {
        return false;
    };
    let Some(to_partition_id) = u64_field(candidate, &["to_partition_id"]) else {
        return false;
    };

    from_partition_id != to_partition_id
        && from_partition_id < expected_partition_count
        && to_partition_id < expected_partition_count
        && positive_u64_field(candidate, &["estimated_event_delta"])
        && string_field(candidate, &["reason"]).is_some_and(|reason| !reason.trim().is_empty())
}

fn status_is_skewed(value: &Value) -> bool {
    string_field(value, &["status"]).is_some_and(|status| status.eq_ignore_ascii_case("skewed"))
}

fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    json_path(value, path).and_then(Value::as_bool)
}

fn string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    json_path(value, path).and_then(Value::as_str)
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn positive_u64_field(value: &Value, path: &[&str]) -> bool {
    u64_field(value, path).is_some_and(|value| value > 0)
}

fn array_field_empty(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
