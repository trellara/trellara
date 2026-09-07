use tokio_postgres::Row;

use crate::{
    CheckpointError, IcebergTableCommitIntent, IcebergTableCommitReceipt, IcebergTableCommitStatus,
    Result,
};

pub(super) fn intent_from_row(row: Row) -> Result<IcebergTableCommitIntent> {
    Ok(IcebergTableCommitIntent {
        dataset_id: row.try_get("dataset_id")?,
        epoch_id: row.try_get("epoch_id")?,
        epoch_commit_id: row.try_get("epoch_commit_id")?,
        lake_table_name: row.try_get("lake_table_name")?,
        relation: row.try_get("relation")?,
        target: row.try_get("target")?,
        table_commit_id: row.try_get("table_commit_id")?,
        manifest_digest: row.try_get("manifest_digest")?,
        file_count: to_usize("file_count", row.try_get("file_count")?)?,
        record_count: to_u64("record_count", row.try_get("record_count")?)?,
        planned_at: row.try_get("planned_at")?,
    })
}

pub(super) fn receipt_from_row(row: Row) -> Result<IcebergTableCommitReceipt> {
    let status: String = row.try_get("status")?;
    Ok(IcebergTableCommitReceipt {
        dataset_id: row.try_get("dataset_id")?,
        epoch_id: row.try_get("epoch_id")?,
        epoch_commit_id: row.try_get("epoch_commit_id")?,
        target: row.try_get("target")?,
        table_commit_id: row.try_get("table_commit_id")?,
        snapshot_id: row.try_get("snapshot_id")?,
        file_count: to_usize("file_count", row.try_get("file_count")?)?,
        record_count: to_u64("record_count", row.try_get("record_count")?)?,
        status: parse_status(&status)?,
        committed_at: row.try_get("committed_at")?,
    })
}

pub(super) fn status_label(status: IcebergTableCommitStatus) -> &'static str {
    match status {
        IcebergTableCommitStatus::Committed => "committed",
        IcebergTableCommitStatus::AlreadyCommitted => "already_committed",
    }
}

fn parse_status(status: &str) -> Result<IcebergTableCommitStatus> {
    match status {
        "committed" => Ok(IcebergTableCommitStatus::Committed),
        "already_committed" => Ok(IcebergTableCommitStatus::AlreadyCommitted),
        other => Err(CheckpointError::Store(format!(
            "invalid Iceberg receipt status loaded from PostgreSQL: {other}"
        ))),
    }
}

pub(super) fn to_i64(field: &'static str, value: usize) -> Result<i64> {
    i64::try_from(value).map_err(|_| numeric_overflow(field))
}

pub(super) fn u64_to_i64(field: &'static str, value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| numeric_overflow(field))
}

fn to_usize(field: &'static str, value: i64) -> Result<usize> {
    usize::try_from(value).map_err(|_| numeric_overflow(field))
}

fn to_u64(field: &'static str, value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| numeric_overflow(field))
}

fn numeric_overflow(field: &'static str) -> CheckpointError {
    CheckpointError::Store(format!(
        "Iceberg commit {field} exceeds PostgreSQL bigint range"
    ))
}
