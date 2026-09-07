use crate::{IcebergRawCdcColumn, IcebergRawCdcColumnType};

pub(super) fn source_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("source_id", IcebergRawCdcColumnType::String, true),
        ("state", IcebergRawCdcColumnType::String, true),
        ("start_lsn", IcebergRawCdcColumnType::String, true),
        ("end_lsn", IcebergRawCdcColumnType::String, true),
        ("transaction_count", IcebergRawCdcColumnType::Long, true),
        ("change_count", IcebergRawCdcColumnType::Long, true),
        ("checksum_rollup", IcebergRawCdcColumnType::Long, true),
        ("lag_reason", IcebergRawCdcColumnType::String, false),
    ])
}

pub(super) fn table_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("relation", IcebergRawCdcColumnType::String, true),
        ("transaction_count", IcebergRawCdcColumnType::Long, true),
        ("change_count", IcebergRawCdcColumnType::Long, true),
        ("checksum_rollup", IcebergRawCdcColumnType::Long, true),
    ])
}

pub(super) fn partition_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("source_id", IcebergRawCdcColumnType::String, true),
        ("partition_id", IcebergRawCdcColumnType::Int, true),
        ("first_commit_lsn", IcebergRawCdcColumnType::String, true),
        ("last_commit_lsn", IcebergRawCdcColumnType::String, true),
        ("transaction_count", IcebergRawCdcColumnType::Long, true),
        ("event_count", IcebergRawCdcColumnType::Long, true),
        ("checksum_rollup", IcebergRawCdcColumnType::Long, true),
    ])
}

pub(super) fn quarantine_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("source_id", IcebergRawCdcColumnType::String, false),
        ("transaction_id", IcebergRawCdcColumnType::String, false),
        ("commit_lsn", IcebergRawCdcColumnType::String, false),
        ("reason", IcebergRawCdcColumnType::String, true),
        ("details", IcebergRawCdcColumnType::String, false),
        ("recovery_command", IcebergRawCdcColumnType::String, false),
    ])
}

pub(super) fn verification_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
        ("epoch_id", IcebergRawCdcColumnType::String, true),
        ("verification_id", IcebergRawCdcColumnType::String, true),
        (
            "input_transaction_count",
            IcebergRawCdcColumnType::Long,
            true,
        ),
        ("input_change_count", IcebergRawCdcColumnType::Long, true),
        (
            "lake_transaction_count",
            IcebergRawCdcColumnType::Long,
            true,
        ),
        ("lake_change_count", IcebergRawCdcColumnType::Long, true),
        ("checksum_status", IcebergRawCdcColumnType::String, true),
        ("completed_at", IcebergRawCdcColumnType::String, true),
    ])
}

pub(super) fn completeness_columns() -> Vec<IcebergRawCdcColumn> {
    columns(&[
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
    ])
}

fn columns(specs: &[(&str, IcebergRawCdcColumnType, bool)]) -> Vec<IcebergRawCdcColumn> {
    specs
        .iter()
        .enumerate()
        .map(
            |(index, (name, column_type, required))| IcebergRawCdcColumn {
                id: i32::try_from(index + 1).expect("metadata schemas fit in i32 field ids"),
                name: (*name).to_string(),
                column_type: *column_type,
                required: *required,
            },
        )
        .collect()
}
