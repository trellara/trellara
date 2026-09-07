use serde_json::Value;

use crate::pilot_evidence_identity_helpers::meaningful_json_string;
use crate::pilot_evidence_json_path::{positive_u64, string_field_eq};

pub(crate) fn row_intents_have_required_fields(value: &Value) -> bool {
    row_intents(value).is_some_and(|rows| {
        !rows.is_empty()
            && rows.iter().all(|row| {
                meaningful_json_string(row, &["source_id"])
                    && meaningful_json_string(row, &["dataset_id"])
                    && meaningful_json_string(row, &["relation"])
                    && meaningful_json_string(row, &["transaction_id"])
                    && meaningful_json_string(row, &["idempotency_key"])
                    && positive_u64(row, &["total_order"])
            })
    })
}

pub(crate) fn row_intents_have_ddl_boundary_metadata(value: &Value) -> bool {
    row_intents(value)
        .is_some_and(|rows| !rows.is_empty() && rows.iter().all(row_has_ddl_boundary_metadata))
}

pub(crate) fn row_intents(value: &Value) -> Option<&Vec<Value>> {
    value.get("row_intents").and_then(Value::as_array)
}

fn row_has_ddl_boundary_metadata(row: &Value) -> bool {
    let before = row
        .get("ddl_schema_fingerprint_before")
        .and_then(Value::as_u64);
    let after = row
        .get("ddl_schema_fingerprint_after")
        .and_then(Value::as_u64);
    positive_u64(row, &["schema_version"])
        && meaningful_json_string(row, &["ddl_barrier_id"])
        && row
            .get("ddl_barrier_id")
            .and_then(Value::as_str)
            .is_some_and(|barrier| barrier.contains(":ddl"))
        && string_field_eq(row, &["ddl_release_gate"], "post_ddl_dml_release")
        && matches!((before, after), (Some(before), Some(after)) if before > 0 && after > before)
}
