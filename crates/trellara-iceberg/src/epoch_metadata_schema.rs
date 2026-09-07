use sha2::{Digest, Sha256};

use crate::{IcebergRawCdcColumn, IcebergRawCdcColumnType, IcebergRawCdcPartitionField};

pub(crate) fn epoch_metadata_columns() -> Vec<IcebergRawCdcColumn> {
    [
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("dataset_id", IcebergRawCdcColumnType::String, true),
        ("state", IcebergRawCdcColumnType::String, true),
        ("policy", IcebergRawCdcColumnType::String, true),
        ("opened_at", IcebergRawCdcColumnType::String, true),
        ("sealed_at", IcebergRawCdcColumnType::String, true),
        ("required_source_count", IcebergRawCdcColumnType::Long, true),
        ("complete_source_count", IcebergRawCdcColumnType::Long, true),
        ("missing_source_count", IcebergRawCdcColumnType::Long, true),
        (
            "quarantined_source_count",
            IcebergRawCdcColumnType::Long,
            true,
        ),
        ("transaction_count", IcebergRawCdcColumnType::Long, true),
        ("change_count", IcebergRawCdcColumnType::Long, true),
        ("checksum_rollup", IcebergRawCdcColumnType::Long, true),
        ("manifest_digest", IcebergRawCdcColumnType::String, true),
        ("iceberg_snapshot_id", IcebergRawCdcColumnType::String, true),
        (
            "raw_table_snapshot_ids_json",
            IcebergRawCdcColumnType::String,
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (name, column_type, required))| IcebergRawCdcColumn {
            id: i32::try_from(index + 1)
                .expect("epoch metadata schema has fewer than i32::MAX columns"),
            name: name.to_string(),
            column_type,
            required,
        },
    )
    .collect()
}

pub(crate) fn epoch_metadata_partition_fields() -> Vec<IcebergRawCdcPartitionField> {
    Vec::new()
}

pub(crate) fn epoch_metadata_schema_fingerprint(
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
