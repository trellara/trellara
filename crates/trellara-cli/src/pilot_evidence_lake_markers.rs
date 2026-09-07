use serde_json::Value;

use crate::pilot_evidence_identity_helpers::meaningful_json_string;

pub(crate) fn lake_spark_marker_present(marker: &str, contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    match marker {
        "verification_status match" => string_field_eq(&value, &["verification_status"], "match"),
        "spark_consumption_allowed true" => {
            bool_field_eq(&value, &["spark_consumption_allowed"], true)
        }
        "spark consumption contract" => spark_consumption_contract_matches(&value),
        "source dataset identity" => source_dataset_identity_present(&value),
        "released spark consumption gate" => spark_consumption_gate_released_after_match(&value),
        "source ack boundary" => source_ack_boundary_present(&value),
        "source count agreement" => source_count_agreement_present(&value),
        "checksum rollup agreement" => checksum_rollup_agreement_present(&value),
        _ => false,
    }
}

fn spark_consumption_contract_matches(value: &Value) -> bool {
    string_field_eq(
        value,
        &["spark_consumption_contract"],
        trellara_lake::LAKE_EPOCH_CONSUMER_GATE_CONTRACT,
    )
}

fn source_dataset_identity_present(value: &Value) -> bool {
    meaningful_json_string(value, &["dataset_id"])
        && value
            .get("source_rows")
            .and_then(Value::as_array)
            .is_some_and(|rows| rows.iter().any(source_row_has_identity))
}

fn source_row_has_identity(row: &Value) -> bool {
    meaningful_json_string(row, &["source_id"])
}

fn spark_consumption_gate_released_after_match(value: &Value) -> bool {
    string_field_eq(value, &["verification_status"], "match")
        && bool_field_eq(value, &["spark_consumption_allowed"], true)
        && json_path(value, &["spark_consumption_gate"])
            .and_then(Value::as_str)
            .is_some_and(|actual| {
                let normalized = actual.to_ascii_lowercase();
                normalized.starts_with("released")
                    && normalized.contains("stream")
                    && normalized.contains("lake")
                    && normalized.contains("proofs match")
            })
}

fn source_ack_boundary_present(value: &Value) -> bool {
    source_ack_boundary_is_durable(value, &["source_ack_boundary"])
        || source_ack_boundary_is_durable(value, &["committer_topology", "source_ack_boundary"])
}

fn source_count_agreement_present(value: &Value) -> bool {
    bool_field_eq(value, &["source_counts_match"], true)
        && source_rows_match_lake_counts(value)
        && numeric_pair_eq(
            value,
            &["stream_required_source_count"],
            &["lake_required_source_count"],
        )
        && numeric_pair_eq(
            value,
            &["stream_complete_source_count"],
            &["lake_complete_source_count"],
        )
        && numeric_pair_eq(
            value,
            &["stream_missing_source_count"],
            &["lake_missing_source_count"],
        )
        && numeric_pair_eq(
            value,
            &["stream_quarantined_source_count"],
            &["lake_quarantined_source_count"],
        )
}

fn source_rows_match_lake_counts(value: &Value) -> bool {
    let Some(source_rows) = value.get("source_rows").and_then(Value::as_array) else {
        return false;
    };
    let complete_count = source_row_state_count(source_rows, "complete");
    let missing_count = source_row_state_count(source_rows, "missing");
    let quarantined_count = source_row_state_count(source_rows, "quarantined");

    source_rows.len() as u64 == u64_field(value, &["lake_required_source_count"]).unwrap_or(0)
        && complete_count == u64_field(value, &["lake_complete_source_count"])
        && missing_count == u64_field(value, &["lake_missing_source_count"])
        && quarantined_count == u64_field(value, &["lake_quarantined_source_count"])
        && source_rows.len() as u64
            == complete_count.unwrap_or(0)
                + missing_count.unwrap_or(0)
                + quarantined_count.unwrap_or(0)
}

fn source_row_state_count(rows: &[Value], expected_state: &str) -> Option<u64> {
    rows.iter()
        .filter(|row| {
            row.get("state")
                .and_then(Value::as_str)
                .is_some_and(|state| state.eq_ignore_ascii_case(expected_state))
        })
        .count()
        .try_into()
        .ok()
}

fn checksum_rollup_agreement_present(value: &Value) -> bool {
    bool_field_eq(value, &["checksum_rollup_match"], true)
        && numeric_pair_eq(
            value,
            &["stream_checksum_rollup"],
            &["lake_checksum_rollup"],
        )
}

fn source_ack_boundary_is_durable(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual == trellara_lake::RAW_CDC_SOURCE_ACK_BOUNDARY)
}

fn bool_field_eq(value: &Value, path: &[&str], expected: bool) -> bool {
    json_path(value, path)
        .and_then(Value::as_bool)
        .is_some_and(|actual| actual == expected)
}

fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn numeric_pair_eq(value: &Value, left_path: &[&str], right_path: &[&str]) -> bool {
    let Some(left) = json_path(value, left_path).and_then(Value::as_u64) else {
        return false;
    };
    json_path(value, right_path)
        .and_then(Value::as_u64)
        .is_some_and(|right| right == left)
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}

fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}
