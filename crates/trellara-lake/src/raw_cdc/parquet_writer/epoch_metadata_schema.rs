use std::collections::HashMap;
use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef};
use parquet::arrow::PARQUET_FIELD_ID_META_KEY;

pub(super) fn raw_cdc_epoch_metadata_parquet_arrow_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        required_string(1, "epoch_id"),
        required_string(2, "dataset_id"),
        required_string(3, "state"),
        required_string(4, "policy"),
        required_string(5, "opened_at"),
        required_string(6, "sealed_at"),
        required_i64(7, "required_source_count"),
        required_i64(8, "complete_source_count"),
        required_i64(9, "missing_source_count"),
        required_i64(10, "quarantined_source_count"),
        required_i64(11, "transaction_count"),
        required_i64(12, "change_count"),
        required_i64(13, "checksum_rollup"),
        required_string(14, "manifest_digest"),
        required_string(15, "iceberg_snapshot_id"),
        required_string(16, "raw_table_snapshot_ids_json"),
    ]))
}

fn required_string(id: i32, name: &str) -> Field {
    field(id, name, DataType::Utf8, false)
}

fn required_i64(id: i32, name: &str) -> Field {
    field(id, name, DataType::Int64, false)
}

fn field(id: i32, name: &str, data_type: DataType, nullable: bool) -> Field {
    Field::new(name, data_type, nullable).with_metadata(HashMap::from([(
        PARQUET_FIELD_ID_META_KEY.to_string(),
        id.to_string(),
    )]))
}
