use std::collections::HashMap;
use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef};
use parquet::arrow::PARQUET_FIELD_ID_META_KEY;

pub const RAW_CDC_PARQUET_FIELD_ID_KEY: &str = PARQUET_FIELD_ID_META_KEY;

pub fn raw_cdc_parquet_arrow_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        required_string(1, "source_id"),
        required_i32(2, "source_bucket"),
        required_string(3, "database_id"),
        required_string(4, "dataset_id"),
        required_string(5, "relation"),
        required_string(6, "transaction_id"),
        required_string(7, "begin_lsn"),
        required_string(8, "commit_lsn"),
        required_i64(9, "commit_timestamp_ms"),
        required_i32(10, "total_order"),
        required_string(11, "operation"),
        optional_string(12, "record_key"),
        required_string(13, "idempotency_key"),
        optional_i64(14, "schema_fingerprint"),
        optional_i64(15, "schema_version"),
        optional_string(16, "ddl_barrier_id"),
        optional_string(17, "ddl_release_gate"),
        optional_i64(18, "ddl_schema_fingerprint_before"),
        optional_i64(19, "ddl_schema_fingerprint_after"),
        required_i64(20, "envelope_checksum"),
        optional_string(21, "manifest_id"),
        optional_string(22, "manifest_boundary_mode"),
        optional_i32(23, "manifest_global_event_count"),
        optional_i32(24, "manifest_participating_partition_count"),
        optional_string(25, "partition_key"),
        required_string(26, "epoch_id"),
        required_string(27, "ingested_at"),
        required_string(28, "payload_before_json"),
        required_string(29, "payload_after_json"),
    ]))
}

fn required_string(id: i32, name: &str) -> Field {
    field(id, name, DataType::Utf8, false)
}

fn optional_string(id: i32, name: &str) -> Field {
    field(id, name, DataType::Utf8, true)
}

fn required_i32(id: i32, name: &str) -> Field {
    field(id, name, DataType::Int32, false)
}

fn optional_i32(id: i32, name: &str) -> Field {
    field(id, name, DataType::Int32, true)
}

fn required_i64(id: i32, name: &str) -> Field {
    field(id, name, DataType::Int64, false)
}

fn optional_i64(id: i32, name: &str) -> Field {
    field(id, name, DataType::Int64, true)
}

fn field(id: i32, name: &str, data_type: DataType, nullable: bool) -> Field {
    Field::new(name, data_type, nullable).with_metadata(HashMap::from([(
        PARQUET_FIELD_ID_META_KEY.to_string(),
        id.to_string(),
    )]))
}
