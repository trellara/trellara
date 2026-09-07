use crate::{
    CheckpointError, IcebergTableCommitIntent, IcebergTableCommitKey, IcebergTableCommitReceipt,
    Result,
};

pub(crate) fn validate_iceberg_commit_key(key: &IcebergTableCommitKey) -> Result<()> {
    validate_clean("dataset_id", &key.dataset_id)?;
    validate_clean("epoch_id", &key.epoch_id)?;
    validate_clean("target", &key.target)?;
    validate_sha256("table_commit_id", &key.table_commit_id)
}

pub(crate) fn validate_iceberg_commit_intent(intent: &IcebergTableCommitIntent) -> Result<()> {
    validate_iceberg_commit_key(&intent.key())?;
    validate_sha256("epoch_commit_id", &intent.epoch_commit_id)?;
    validate_clean("lake_table_name", &intent.lake_table_name)?;
    validate_clean("relation", &intent.relation)?;
    validate_sha256("manifest_digest", &intent.manifest_digest)?;
    validate_non_zero("file_count", intent.file_count)?;
    validate_non_zero_u64("record_count", intent.record_count)?;
    validate_clean("planned_at", &intent.planned_at)
}

pub(crate) fn validate_iceberg_commit_receipt(receipt: &IcebergTableCommitReceipt) -> Result<()> {
    validate_iceberg_commit_key(&receipt.key())?;
    validate_sha256("epoch_commit_id", &receipt.epoch_commit_id)?;
    validate_non_zero("file_count", receipt.file_count)?;
    validate_non_zero_u64("record_count", receipt.record_count)?;
    if receipt.snapshot_id <= 0 {
        return invalid("snapshot_id", "must be greater than zero");
    }
    validate_clean("committed_at", &receipt.committed_at)
}

pub(crate) fn validate_iceberg_epoch_lookup(dataset_id: &str, epoch_id: &str) -> Result<()> {
    validate_clean("dataset_id", dataset_id)?;
    validate_clean("epoch_id", epoch_id)
}

pub(crate) fn ensure_same_intent(
    existing: &IcebergTableCommitIntent,
    incoming: &IcebergTableCommitIntent,
) -> Result<()> {
    // `planned_at` is observation metadata, not commit identity. A process may
    // restart after persisting an intent but before receiving a catalog
    // receipt, and must be able to resume the exact same commit with a new
    // attempt timestamp.
    if existing.dataset_id == incoming.dataset_id
        && existing.epoch_id == incoming.epoch_id
        && existing.epoch_commit_id == incoming.epoch_commit_id
        && existing.lake_table_name == incoming.lake_table_name
        && existing.relation == incoming.relation
        && existing.target == incoming.target
        && existing.table_commit_id == incoming.table_commit_id
        && existing.manifest_digest == incoming.manifest_digest
        && existing.file_count == incoming.file_count
        && existing.record_count == incoming.record_count
    {
        Ok(())
    } else {
        Err(CheckpointError::Store(format!(
            "conflicting Iceberg commit intent for {}",
            existing.key().target
        )))
    }
}

pub(crate) fn ensure_same_receipt(
    existing: &IcebergTableCommitReceipt,
    incoming: &IcebergTableCommitReceipt,
) -> Result<()> {
    // Concurrent callers can observe the same durable snapshot differently:
    // one receives the commit response while another reconciles it as already
    // committed. Status and attempt timestamp are therefore not receipt
    // identity; the snapshot and deterministic commit evidence are.
    if existing.dataset_id == incoming.dataset_id
        && existing.epoch_id == incoming.epoch_id
        && existing.epoch_commit_id == incoming.epoch_commit_id
        && existing.target == incoming.target
        && existing.table_commit_id == incoming.table_commit_id
        && existing.snapshot_id == incoming.snapshot_id
        && existing.file_count == incoming.file_count
        && existing.record_count == incoming.record_count
    {
        Ok(())
    } else {
        Err(CheckpointError::Store(format!(
            "conflicting Iceberg commit receipt for {}",
            existing.key().target
        )))
    }
}

fn validate_clean(field: &'static str, value: &str) -> Result<()> {
    if value.trim() == value && !value.is_empty() && !value.chars().any(char::is_control) {
        Ok(())
    } else {
        invalid(
            field,
            "must be non-empty and contain no surrounding whitespace or controls",
        )
    }
}

fn validate_sha256(field: &'static str, value: &str) -> Result<()> {
    validate_clean(field, value)?;
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(())
    } else {
        invalid(field, "must be a 64-character SHA-256 hex digest")
    }
}

fn validate_non_zero(field: &'static str, value: usize) -> Result<()> {
    if value > 0 {
        Ok(())
    } else {
        invalid(field, "must be greater than zero")
    }
}

fn validate_non_zero_u64(field: &'static str, value: u64) -> Result<()> {
    if value > 0 {
        Ok(())
    } else {
        invalid(field, "must be greater than zero")
    }
}

fn invalid<T>(field: &'static str, reason: &str) -> Result<T> {
    Err(CheckpointError::Store(format!(
        "invalid Iceberg commit {field}: {reason}"
    )))
}
