use crate::{
    manifest_affected_tables::validate_affected_table_event_counts, parse_lsn,
    validate_manifest_partition_ids, ManifestBoundaryMode, ProtocolError, TransactionManifest,
};

pub fn validate_transaction_manifest(manifest: &TransactionManifest) -> Result<(), ProtocolError> {
    validate_manifest_identity(manifest)?;
    validate_manifest_boundary_mode(manifest)?;
    validate_manifest_partition_ids(manifest)?;
    validate_manifest_event_counts(manifest)
}

pub(crate) fn validate_manifest_event_counts(
    manifest: &TransactionManifest,
) -> Result<(), ProtocolError> {
    validate_manifest_partitions(manifest)?;
    validate_affected_table_event_counts(manifest)
}

fn validate_manifest_partitions(manifest: &TransactionManifest) -> Result<(), ProtocolError> {
    if manifest.partitions.is_empty() {
        return invalid_manifest_field("partitions", "must contain at least one partition");
    }

    let partition_event_count = manifest
        .partitions
        .iter()
        .try_fold(0u32, |total, partition| {
            if partition.first_total_order == 0 {
                return invalid_manifest_field(
                    "partitions[].first_total_order",
                    "must be greater than zero",
                );
            }
            if partition.last_total_order == 0 {
                return invalid_manifest_field(
                    "partitions[].last_total_order",
                    "must be greater than zero",
                );
            }
            if partition.first_total_order > partition.last_total_order {
                return invalid_manifest_field(
                    "partitions[].total_order_range",
                    "first_total_order must be less than or equal to last_total_order",
                );
            }
            let total_order_span = partition.last_total_order - partition.first_total_order + 1;
            if partition.event_count == 0 {
                return invalid_manifest_field(
                    "partitions[].event_count",
                    "must be greater than zero",
                );
            }
            if partition.event_count > total_order_span {
                return invalid_manifest_field(
                    "partitions[].event_count",
                    "must not exceed the declared total_order range span",
                );
            }
            if partition.last_total_order > manifest.global_event_count {
                return invalid_manifest_field(
                    "partitions[].last_total_order",
                    "must not exceed global_event_count",
                );
            }
            total.checked_add(partition.event_count).ok_or_else(|| {
                ProtocolError::ManifestCountOverflow {
                    transaction_id: manifest.transaction_id.clone(),
                    field: "global_event_count",
                    max_supported_count: u32::MAX,
                }
            })
        })?;

    if partition_event_count != manifest.global_event_count {
        return Err(ProtocolError::ManifestEventCountMismatch {
            transaction_id: manifest.transaction_id.clone(),
            expected: manifest.global_event_count,
            actual: partition_event_count,
        });
    }
    validate_manifest_includes_first_total_order(manifest)?;

    Ok(())
}

fn validate_manifest_includes_first_total_order(
    manifest: &TransactionManifest,
) -> Result<(), ProtocolError> {
    let lowest_first_total_order = manifest
        .partitions
        .iter()
        .map(|partition| partition.first_total_order)
        .min()
        .expect("manifest partitions are non-empty");
    if lowest_first_total_order != 1 {
        return invalid_manifest_field(
            "partitions[].first_total_order",
            "manifest must include the first transaction event",
        );
    }

    Ok(())
}

fn validate_manifest_identity(manifest: &TransactionManifest) -> Result<(), ProtocolError> {
    require_manifest_field("transaction_id", &manifest.transaction_id)?;
    require_manifest_field("source_commit_lsn", &manifest.source_commit_lsn)?;
    let parsed = parse_lsn(&manifest.source_commit_lsn)?;
    if parsed == 0 {
        return invalid_manifest_field("source_commit_lsn", "must be greater than zero");
    }
    if manifest.global_event_count == 0 {
        return invalid_manifest_field("global_event_count", "must be greater than zero");
    }
    if manifest.source_commit_timestamp_ms <= 0 {
        return invalid_manifest_field("source_commit_timestamp_ms", "must be greater than zero");
    }
    Ok(())
}

fn validate_manifest_boundary_mode(manifest: &TransactionManifest) -> Result<(), ProtocolError> {
    match ManifestBoundaryMode::try_from(manifest.boundary_mode) {
        Ok(
            ManifestBoundaryMode::StrictChunkedTransactionOrder
            | ManifestBoundaryMode::PartitionedScale,
        ) => Ok(()),
        Ok(ManifestBoundaryMode::Unspecified) | Err(_) => invalid_manifest_field(
            "boundary_mode",
            "must be strict_chunked_transaction_order or partitioned_scale_mode",
        ),
    }
}

fn require_manifest_field(field: &'static str, value: &str) -> Result<(), ProtocolError> {
    if value.trim().is_empty() {
        return invalid_manifest_field(field, "must not be empty");
    }
    if value != value.trim() {
        return invalid_manifest_field(field, "must not contain surrounding whitespace");
    }
    Ok(())
}

fn invalid_manifest_field<T>(
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, ProtocolError> {
    Err(ProtocolError::InvalidManifestField {
        field,
        reason: reason.into(),
    })
}
