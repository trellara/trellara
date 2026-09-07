use trellara_protocol::{
    validate_manifest_partition_ids, validate_transaction_manifest, ManifestBoundaryMode,
    TransactionEnvelope,
};

use crate::commit_manifest_orders::validate_manifest_order_boundaries;
use crate::LakeError;

pub(crate) fn validate_epoch_manifest(envelope: &TransactionEnvelope) -> Result<(), LakeError> {
    if let Some(manifest) = &envelope.manifest {
        validated_manifest_boundary_mode(manifest.boundary_mode)?;
        validate_manifest_partition_ids(manifest)?;
        validate_manifest_boundary(
            "transaction_id",
            &envelope.transaction_id,
            &manifest.transaction_id,
        )?;
        validate_manifest_boundary(
            "commit_lsn",
            &envelope.commit_lsn,
            &manifest.source_commit_lsn,
        )?;
        validate_manifest_boundary(
            "commit_timestamp_ms",
            &envelope.commit_timestamp_ms.to_string(),
            &manifest.source_commit_timestamp_ms.to_string(),
        )?;
        let actual = manifest_count("envelope.changes", envelope.changes.len())?;
        if manifest.global_event_count != actual {
            return Err(LakeError::ManifestEventCountMismatch {
                expected: manifest.global_event_count,
                actual,
            });
        }
        let partition_event_count =
            manifest
                .partitions
                .iter()
                .try_fold(0u32, |total, partition| {
                    total.checked_add(partition.event_count).ok_or(
                        LakeError::ManifestCountOverflow {
                            field: "partition.event_count",
                            max_supported_count: u32::MAX,
                        },
                    )
                })?;
        if partition_event_count != manifest.global_event_count {
            return Err(LakeError::ManifestEventCountMismatch {
                expected: manifest.global_event_count,
                actual: partition_event_count,
            });
        }
        validate_manifest_order_boundaries(envelope)?;
        validate_transaction_manifest(manifest)?;
    }
    Ok(())
}

pub(crate) fn validated_manifest_boundary_mode(
    boundary_mode: i32,
) -> Result<ManifestBoundaryMode, LakeError> {
    match ManifestBoundaryMode::try_from(boundary_mode) {
        Ok(
            mode @ (ManifestBoundaryMode::StrictChunkedTransactionOrder
            | ManifestBoundaryMode::PartitionedScale),
        ) => Ok(mode),
        Ok(ManifestBoundaryMode::Unspecified) | Err(_) => {
            Err(LakeError::InvalidManifestBoundaryMode { boundary_mode })
        }
    }
}

pub(crate) fn manifest_count(field: &'static str, count: usize) -> Result<u32, LakeError> {
    u32::try_from(count).map_err(|_| LakeError::ManifestCountOverflow {
        field,
        max_supported_count: u32::MAX,
    })
}

fn validate_manifest_boundary(
    field: &'static str,
    envelope: &str,
    manifest: &str,
) -> Result<(), LakeError> {
    if envelope == manifest {
        Ok(())
    } else {
        Err(LakeError::ManifestBoundaryMismatch {
            field,
            envelope: envelope.to_string(),
            manifest: manifest.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_count_fails_closed_before_wraparound() {
        assert!(matches!(
            manifest_count("chunk_count", u32::MAX as usize + 1),
            Err(LakeError::ManifestCountOverflow {
                field: "chunk_count",
                max_supported_count: u32::MAX,
            })
        ));
    }
}
