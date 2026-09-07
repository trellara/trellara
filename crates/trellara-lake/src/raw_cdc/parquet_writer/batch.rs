use std::sync::Arc;

use arrow_array::{ArrayRef, Int32Array, Int64Array, RecordBatch, StringArray};

use crate::{LakeColumnValue, LakeError, LakeRawCdcRowIntent};

use super::schema::raw_cdc_parquet_arrow_schema;

pub(super) fn record_batch(rows: &[&LakeRawCdcRowIntent]) -> Result<RecordBatch, LakeError> {
    let source_buckets = rows
        .iter()
        .map(|row| i32_from_u32("source_bucket", row.source_bucket))
        .collect::<Result<Vec<_>, _>>()?;
    let total_orders = rows
        .iter()
        .map(|row| i32_from_u32("total_order", row.total_order))
        .collect::<Result<Vec<_>, _>>()?;
    let manifest_global_counts = rows
        .iter()
        .map(|row| {
            option_i32_from_u32(
                "manifest_global_event_count",
                row.manifest_global_event_count,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let manifest_partition_counts = rows
        .iter()
        .map(|row| {
            option_i32_from_u32(
                "manifest_participating_partition_count",
                row.manifest_participating_partition_count,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payload_before_json = payload_json(rows, |row| &row.payload_before, "payload_before")?;
    let payload_after_json = payload_json(rows, |row| &row.payload_after, "payload_after")?;

    RecordBatch::try_new(
        raw_cdc_parquet_arrow_schema(),
        vec![
            string_values(rows.iter().map(|row| row.source_id.as_str())),
            Arc::new(Int32Array::from(source_buckets)) as ArrayRef,
            string_values(rows.iter().map(|row| row.database_id.as_str())),
            string_values(rows.iter().map(|row| row.dataset_id.as_str())),
            string_values(rows.iter().map(|row| row.relation.as_str())),
            string_values(rows.iter().map(|row| row.transaction_id.as_str())),
            string_values(rows.iter().map(|row| row.begin_lsn.as_str())),
            string_values(rows.iter().map(|row| row.commit_lsn.as_str())),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.commit_timestamp_ms),
            )) as ArrayRef,
            Arc::new(Int32Array::from(total_orders)) as ArrayRef,
            string_values(rows.iter().map(|row| row.operation.as_str())),
            optional_string_values(rows.iter().map(|row| row.record_key.as_deref())),
            string_values(rows.iter().map(|row| row.idempotency_key.as_str())),
            optional_i64_values(rows.iter().map(|row| row.schema_fingerprint)),
            optional_i64_values(rows.iter().map(|row| row.schema_version)),
            optional_string_values(rows.iter().map(|row| row.ddl_barrier_id.as_deref())),
            optional_string_values(rows.iter().map(|row| row.ddl_release_gate.as_deref())),
            optional_i64_values(rows.iter().map(|row| row.ddl_schema_fingerprint_before)),
            optional_i64_values(rows.iter().map(|row| row.ddl_schema_fingerprint_after)),
            i64_values(rows.iter().map(|row| row.envelope_checksum)),
            optional_string_values(rows.iter().map(|row| row.manifest_id.as_deref())),
            optional_string_values(rows.iter().map(|row| row.manifest_boundary_mode.as_deref())),
            Arc::new(Int32Array::from(manifest_global_counts)) as ArrayRef,
            Arc::new(Int32Array::from(manifest_partition_counts)) as ArrayRef,
            optional_string_values(rows.iter().map(|row| row.partition_key.as_deref())),
            string_values(rows.iter().map(|row| row.epoch_id.as_str())),
            string_values(rows.iter().map(|row| row.ingested_at.as_str())),
            string_values(payload_before_json.iter().map(String::as_str)),
            string_values(payload_after_json.iter().map(String::as_str)),
        ],
    )
    .map_err(|error| LakeError::RawCdcParquet {
        operation: "build_record_batch",
        message: error.to_string(),
    })
}

fn payload_json(
    rows: &[&LakeRawCdcRowIntent],
    select: fn(&LakeRawCdcRowIntent) -> &[LakeColumnValue],
    field: &'static str,
) -> Result<Vec<String>, LakeError> {
    rows.iter()
        .map(|row| json_payload(field, select(row)))
        .collect()
}

fn string_values<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn optional_string_values<'a>(values: impl IntoIterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from_iter(values)) as ArrayRef
}

fn i64_values(values: impl IntoIterator<Item = u64>) -> ArrayRef {
    Arc::new(Int64Array::from_iter_values(
        values.into_iter().map(i64_bit_pattern_from_u64),
    )) as ArrayRef
}

fn optional_i64_values(values: impl IntoIterator<Item = Option<u64>>) -> ArrayRef {
    Arc::new(Int64Array::from_iter(
        values
            .into_iter()
            .map(|value| value.map(i64_bit_pattern_from_u64)),
    )) as ArrayRef
}

fn json_payload(field: &'static str, payload: &[LakeColumnValue]) -> Result<String, LakeError> {
    serde_json::to_string(payload).map_err(|error| LakeError::RawCdcParquet {
        operation: field,
        message: error.to_string(),
    })
}

fn i32_from_u32(field: &'static str, value: u32) -> Result<i32, LakeError> {
    i32::try_from(value).map_err(|_| LakeError::RawCdcParquetCountOverflow { field })
}

fn option_i32_from_u32(field: &'static str, value: Option<u32>) -> Result<Option<i32>, LakeError> {
    value.map(|value| i32_from_u32(field, value)).transpose()
}

fn i64_bit_pattern_from_u64(value: u64) -> i64 {
    i64::from_ne_bytes(value.to_ne_bytes())
}
