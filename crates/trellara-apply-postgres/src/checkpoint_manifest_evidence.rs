use trellara_protocol::{
    validate_transaction_manifest, ManifestBoundaryMode, ProtocolError, TransactionEnvelope,
    TransactionManifest,
};

use crate::{ApplyError, Result};

pub(crate) fn validate_checkpoint_manifest(
    envelope: &TransactionEnvelope,
    manifest: &TransactionManifest,
) -> Result<()> {
    validate_manifest_boundary(envelope, manifest)?;
    if manifest.partitions.is_empty() {
        return Err(ApplyError::Protocol(ProtocolError::InvalidPartitionCount));
    }
    validate_manifest_boundary_mode(manifest.boundary_mode)?;
    validate_transaction_manifest(manifest)?;
    Ok(())
}

fn validate_manifest_boundary(
    envelope: &TransactionEnvelope,
    manifest: &TransactionManifest,
) -> Result<()> {
    if manifest.transaction_id != envelope.transaction_id {
        return Err(ApplyError::CheckpointBoundaryMismatch {
            field: "transaction_id",
            envelope: envelope.transaction_id.clone(),
            manifest: manifest.transaction_id.clone(),
        });
    }
    if manifest.source_commit_lsn != envelope.commit_lsn {
        return Err(ApplyError::CheckpointBoundaryMismatch {
            field: "commit_lsn",
            envelope: envelope.commit_lsn.clone(),
            manifest: manifest.source_commit_lsn.clone(),
        });
    }
    Ok(())
}

fn validate_manifest_boundary_mode(boundary_mode: i32) -> Result<()> {
    match ManifestBoundaryMode::try_from(boundary_mode) {
        Ok(
            ManifestBoundaryMode::StrictChunkedTransactionOrder
            | ManifestBoundaryMode::PartitionedScale,
        ) => Ok(()),
        Ok(ManifestBoundaryMode::Unspecified) | Err(_) => {
            Err(ApplyError::InvalidCheckpointManifestBoundaryMode { boundary_mode })
        }
    }
}
