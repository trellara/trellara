use std::collections::BTreeSet;

use serde_json::Value;

#[path = "pilot_evidence_partition_consistency.rs"]
mod pilot_evidence_partition_consistency;
#[path = "pilot_evidence_partition_health.rs"]
mod pilot_evidence_partition_health;
#[path = "pilot_evidence_partition_json.rs"]
mod pilot_evidence_partition_json;

use crate::pilot_evidence_identity_helpers::source_dataset_identity_present as json_source_dataset_identity_present;
use pilot_evidence_partition_health::partition_scale_health_ready;
use pilot_evidence_partition_json::{
    array_field_empty, bool_field, json_path, lsn_field_at_or_after, lsn_field_valid, u64_field,
};

pub(crate) fn partition_watermark_marker_present(marker: &str, contents: &str) -> bool {
    let Some(value) = serde_json::from_str::<Value>(contents).ok() else {
        return false;
    };

    match marker {
        "source dataset identity" => source_dataset_identity_present(&value),
        "complete_partition_set true" => partition_watermarks_complete(&value),
        "partition_scale_health ready" => partition_scale_health_ready(&value),
        _ => false,
    }
}

fn source_dataset_identity_present(value: &Value) -> bool {
    json_source_dataset_identity_present(value, &["source_id"], &["dataset_id"])
}

fn partition_watermarks_complete(value: &Value) -> bool {
    let expected = u64_field(value, &["expected_partition_count"]);
    let observed = u64_field(value, &["observed_partition_count"]);

    bool_field(value, &["complete_partition_set"]).is_some_and(|complete| complete)
        && expected.is_some_and(|count| count > 0)
        && expected == observed
        && lsn_field_valid(value, &["global_durable_lsn"])
        && lsn_field_valid(value, &["global_applied_lsn"])
        && lsn_field_at_or_after(value, &["global_durable_lsn"], &["global_applied_lsn"])
        && array_field_empty(value, &["missing_partitions"])
        && partitions_match_expected(value, expected)
}

fn partitions_match_expected(value: &Value, expected: Option<u64>) -> bool {
    let Some(partitions) = json_path(value, &["partitions"]).and_then(Value::as_array) else {
        return false;
    };

    expected.is_some_and(|count| partitions.len() as u64 == count)
        && partition_ids_cover_expected_set(partitions, expected)
        && partitions.iter().all(|partition| {
            !bool_field(partition, &["blocks_global_applied_watermark"]).unwrap_or(true)
                && u64_field(partition, &["partition_id"]).is_some()
                && lsn_field_valid(partition, &["last_durable_lsn"])
                && lsn_field_valid(partition, &["last_applied_lsn"])
                && lsn_field_at_or_after(partition, &["last_durable_lsn"], &["last_applied_lsn"])
        })
}

fn partition_ids_cover_expected_set(partitions: &[Value], expected: Option<u64>) -> bool {
    let Some(expected) = expected else {
        return false;
    };
    let ids = partitions
        .iter()
        .filter_map(|partition| u64_field(partition, &["partition_id"]))
        .collect::<BTreeSet<_>>();

    ids.len() as u64 == expected && ids.iter().copied().eq(0..expected)
}
