use serde_json::Value;
use std::collections::HashSet;

#[path = "pilot_evidence_lake_writer_replay.rs"]
mod pilot_evidence_lake_writer_replay;

use crate::pilot_evidence_identity_helpers::meaningful_json_string;
use crate::pilot_evidence_json_path::{field_contains, json_path, json_value, string_field_eq};
use crate::pilot_evidence_lake_writer_rows::{
    row_intents, row_intents_have_ddl_boundary_metadata, row_intents_have_required_fields,
};
use pilot_evidence_lake_writer_replay::duplicate_replay_accounting_present;

pub(crate) fn lake_writer_marker_present(marker: &str, contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    match marker {
        "source dataset identity" => source_dataset_identity_present(&value),
        "raw CDC row intents" => row_intents_have_required_fields(&value),
        "partition source evidence" => partition_source_evidence_present(&value),
        "commit ordering" => commit_steps_are_ordered(&value),
        "durability gates" => durability_gates_present(&value),
        "duplicate replay accounting" => duplicate_replay_accounting_present(&value),
        "Iceberg checkpoint receipt gate" => iceberg_checkpoint_receipt_gate_present(&value),
        "DDL boundary metadata" => row_intents_have_ddl_boundary_metadata(&value),
        _ => false,
    }
}

fn source_dataset_identity_present(value: &Value) -> bool {
    let Some(dataset_id) = value.get("dataset_id").and_then(Value::as_str) else {
        return false;
    };
    meaningful_json_string(value, &["dataset_id"])
        && row_intents(value).is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("dataset_id").and_then(Value::as_str) == Some(dataset_id)
                    && meaningful_json_string(row, &["source_id"])
            })
        })
}

fn partition_source_evidence_present(value: &Value) -> bool {
    let Some(source_ids) = epoch_source_ids(value) else {
        return false;
    };
    epoch_partition_rows(value).is_some_and(|rows| {
        rows.iter().all(|partition| {
            partition
                .get("source_id")
                .and_then(Value::as_str)
                .is_some_and(|source_id| source_ids.contains(source_id))
        })
    })
}

fn epoch_source_ids(value: &Value) -> Option<HashSet<&str>> {
    Some(
        epoch_source_rows(value)?
            .iter()
            .filter_map(|source| source.get("source_id").and_then(Value::as_str))
            .collect(),
    )
    .filter(|source_ids: &HashSet<&str>| !source_ids.is_empty())
}

fn epoch_source_rows(value: &Value) -> Option<&Vec<Value>> {
    json_path(value, &["epoch_metadata", "source_rows"]).and_then(Value::as_array)
}

fn epoch_partition_rows(value: &Value) -> Option<&Vec<Value>> {
    json_path(value, &["epoch_metadata", "partition_rows"]).and_then(Value::as_array)
}

fn commit_steps_are_ordered(value: &Value) -> bool {
    let Some(steps) = value.get("commit_steps").and_then(Value::as_array) else {
        return false;
    };
    let raw = commit_step_order(steps, "raw_cdc_data");
    let epoch = commit_step_order(steps, "epoch_row_metadata");
    let verification = commit_step_order(steps, "verification_metadata");
    matches!((raw, epoch, verification), (Some(raw), Some(epoch), Some(verification)) if raw < epoch && epoch < verification)
}

fn durability_gates_present(value: &Value) -> bool {
    source_ack_boundary_is_durable(value)
        && value
            .get("commit_steps")
            .and_then(Value::as_array)
            .is_some_and(|steps| steps.iter().all(commit_step_has_durability_gate))
}

fn iceberg_checkpoint_receipt_gate_present(value: &Value) -> bool {
    metadata_commit_steps(value).is_some_and(|steps| {
        !steps.is_empty()
            && steps
                .iter()
                .all(|step| field_contains(step, &["durability_gate"], "checkpoint receipts"))
    }) && value
        .get("recovery_scenarios")
        .and_then(Value::as_array)
        .is_some_and(|scenarios| {
            scenarios.iter().any(|scenario| {
                field_contains(scenario, &["recovery_action"], "checkpoint receipts")
                    && field_contains(scenario, &["recovery_action"], "epoch metadata")
            })
        })
}

fn metadata_commit_steps(value: &Value) -> Option<Vec<&Value>> {
    value
        .get("commit_steps")
        .and_then(Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .filter(|step| {
                    [
                        "epoch_sources_metadata",
                        "epoch_tables_metadata",
                        "epoch_row_metadata",
                        "verification_metadata",
                    ]
                    .iter()
                    .any(|phase| string_field_eq(step, &["phase"], phase))
                })
                .collect()
        })
}

fn commit_step_order(steps: &[Value], phase: &str) -> Option<u64> {
    steps.iter().find_map(|step| {
        if string_field_eq(step, &["phase"], phase) {
            step.get("order").and_then(Value::as_u64)
        } else {
            None
        }
    })
}

fn commit_step_has_durability_gate(step: &Value) -> bool {
    let Some(gate) = step.get("durability_gate").and_then(Value::as_str) else {
        return false;
    };
    let normalized = gate.to_ascii_lowercase();
    normalized.contains("before source acknowledgement")
        || normalized.contains("checkpoint receipts")
}

fn source_ack_boundary_is_durable(value: &Value) -> bool {
    json_path(value, &["committer_topology", "source_ack_boundary"])
        .or_else(|| json_path(value, &["source_ack_boundary"]))
        .and_then(Value::as_str)
        .is_some_and(|actual| actual == trellara_lake::RAW_CDC_SOURCE_ACK_BOUNDARY)
}
