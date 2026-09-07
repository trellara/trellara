use sha2::{Digest, Sha256};

use crate::{IcebergRawCdcColumn, IcebergRawCdcColumnType, IcebergRawCdcPartitionField};

pub(crate) fn raw_cdc_columns() -> Vec<IcebergRawCdcColumn> {
    [
        ("source_id", IcebergRawCdcColumnType::String, true),
        ("source_bucket", IcebergRawCdcColumnType::Int, true),
        ("database_id", IcebergRawCdcColumnType::String, true),
        ("dataset_id", IcebergRawCdcColumnType::String, true),
        ("relation", IcebergRawCdcColumnType::String, true),
        ("transaction_id", IcebergRawCdcColumnType::String, true),
        ("begin_lsn", IcebergRawCdcColumnType::String, true),
        ("commit_lsn", IcebergRawCdcColumnType::String, true),
        ("commit_timestamp_ms", IcebergRawCdcColumnType::Long, true),
        ("total_order", IcebergRawCdcColumnType::Int, true),
        ("operation", IcebergRawCdcColumnType::String, true),
        ("record_key", IcebergRawCdcColumnType::String, false),
        ("idempotency_key", IcebergRawCdcColumnType::String, true),
        ("schema_fingerprint", IcebergRawCdcColumnType::Long, false),
        ("schema_version", IcebergRawCdcColumnType::Long, false),
        ("ddl_barrier_id", IcebergRawCdcColumnType::String, false),
        ("ddl_release_gate", IcebergRawCdcColumnType::String, false),
        (
            "ddl_schema_fingerprint_before",
            IcebergRawCdcColumnType::Long,
            false,
        ),
        (
            "ddl_schema_fingerprint_after",
            IcebergRawCdcColumnType::Long,
            false,
        ),
        ("envelope_checksum", IcebergRawCdcColumnType::Long, true),
        ("manifest_id", IcebergRawCdcColumnType::String, false),
        (
            "manifest_boundary_mode",
            IcebergRawCdcColumnType::String,
            false,
        ),
        (
            "manifest_global_event_count",
            IcebergRawCdcColumnType::Int,
            false,
        ),
        (
            "manifest_participating_partition_count",
            IcebergRawCdcColumnType::Int,
            false,
        ),
        ("partition_key", IcebergRawCdcColumnType::String, false),
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("ingested_at", IcebergRawCdcColumnType::String, true),
        ("payload_before_json", IcebergRawCdcColumnType::String, true),
        ("payload_after_json", IcebergRawCdcColumnType::String, true),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (name, column_type, required))| IcebergRawCdcColumn {
            id: i32::try_from(index + 1).expect("raw CDC schema has fewer than i32::MAX columns"),
            name: name.to_string(),
            column_type,
            required,
        },
    )
    .collect()
}

pub(crate) fn raw_cdc_partition_fields() -> Vec<IcebergRawCdcPartitionField> {
    vec![
        IcebergRawCdcPartitionField::identity(26, "epoch_id"),
        IcebergRawCdcPartitionField::identity(2, "source_bucket"),
    ]
}

pub(crate) fn schema_fingerprint(
    columns: &[IcebergRawCdcColumn],
    partition_fields: &[IcebergRawCdcPartitionField],
) -> String {
    let mut hasher = Sha256::new();
    for column in columns {
        hasher.update(column.id.to_string().as_bytes());
        hasher.update(b"|");
        hasher.update(column.name.as_bytes());
        hasher.update(b"|");
        hasher.update(column.column_type.label().as_bytes());
        hasher.update(b"|");
        hasher.update(column.required.to_string().as_bytes());
        hasher.update(b"\n");
    }
    for field in partition_fields {
        hasher.update(field.source_column_id.to_string().as_bytes());
        hasher.update(b"|");
        hasher.update(field.source_column.as_bytes());
        hasher.update(b"|");
        hasher.update(field.transform.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}
