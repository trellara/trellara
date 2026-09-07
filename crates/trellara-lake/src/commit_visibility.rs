use trellara_protocol::{ManifestBoundaryMode, TransactionEnvelope};

use crate::commit_manifest_validation::{
    manifest_count, validate_epoch_manifest, validated_manifest_boundary_mode,
};
use crate::{LakeError, LakeVisibilityBoundary};

pub(crate) fn visibility_boundary(
    envelope: &TransactionEnvelope,
) -> Result<LakeVisibilityBoundary, LakeError> {
    let Some(manifest) = &envelope.manifest else {
        return Ok(LakeVisibilityBoundary::StrictEnvelope);
    };
    validate_epoch_manifest(envelope)?;

    match validated_manifest_boundary_mode(manifest.boundary_mode)? {
        ManifestBoundaryMode::StrictChunkedTransactionOrder => {
            Ok(LakeVisibilityBoundary::StrictChunkManifestBarrier {
                global_event_count: manifest.global_event_count,
                chunk_count: manifest_count("chunk_count", manifest.partitions.len())?,
            })
        }
        ManifestBoundaryMode::PartitionedScale => {
            Ok(LakeVisibilityBoundary::PartitionManifestBarrier {
                global_event_count: manifest.global_event_count,
                participating_partition_count: manifest_count(
                    "participating_partition_count",
                    manifest.partitions.len(),
                )?,
            })
        }
        ManifestBoundaryMode::Unspecified => unreachable!("validated manifest boundary mode"),
    }
}
