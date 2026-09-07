use serde_json::Value;

use super::pilot_evidence_partition_consistency::{string_fields_match, u64_fields_match};
use super::pilot_evidence_partition_json::{
    array_field_empty, bool_field, lsn_field_at_or_after, lsn_field_valid, lsn_fields_match,
    string_field,
};

pub(super) fn partition_scale_health_ready(value: &Value) -> bool {
    string_field(value, &["partition_scale_health", "status"])
        .is_some_and(|status| status.eq_ignore_ascii_case("Ready"))
        && bool_field(
            value,
            &["partition_scale_health", "global_watermark_available"],
        )
        .is_some_and(|available| available)
        && bool_field(
            value,
            &["partition_scale_health", "global_visibility_releasable"],
        )
        .is_some_and(|releasable| releasable)
        && array_field_empty(value, &["partition_scale_health", "blocking_partition_ids"])
        && array_field_empty(value, &["partition_scale_health", "lagging_partition_ids"])
        && array_field_empty(
            value,
            &["partition_scale_health", "straggler_partition_ids"],
        )
        && array_field_empty(value, &["partition_scale_health", "missing_partitions"])
        && string_fields_match(
            value,
            &["partition_scale_health", "source_id"],
            &["source_id"],
        )
        && string_fields_match(
            value,
            &["partition_scale_health", "dataset_id"],
            &["dataset_id"],
        )
        && u64_fields_match(
            value,
            &["partition_scale_health", "expected_partition_count"],
            &["expected_partition_count"],
        )
        && u64_fields_match(
            value,
            &["partition_scale_health", "observed_partition_count"],
            &["observed_partition_count"],
        )
        && lsn_field_valid(value, &["partition_scale_health", "global_durable_lsn"])
        && lsn_field_valid(value, &["partition_scale_health", "global_applied_lsn"])
        && lsn_fields_match(
            value,
            &["partition_scale_health", "global_durable_lsn"],
            &["global_durable_lsn"],
        )
        && lsn_fields_match(
            value,
            &["partition_scale_health", "global_applied_lsn"],
            &["global_applied_lsn"],
        )
        && lsn_field_at_or_after(
            value,
            &["partition_scale_health", "global_durable_lsn"],
            &["partition_scale_health", "global_applied_lsn"],
        )
        && lsn_fields_match(
            value,
            &["partition_scale_health", "global_durable_lsn"],
            &["partition_scale_health", "global_applied_lsn"],
        )
}
