use trellara_protocol::TransactionManifest;

use crate::commit_manifest_validation::{manifest_count, validated_manifest_boundary_mode};
use crate::LakeError;

pub(super) fn manifest_id(manifest: &TransactionManifest) -> String {
    format!(
        "{}:{}:{}",
        manifest.transaction_id, manifest.source_commit_lsn, manifest.boundary_mode
    )
}

pub(super) fn manifest_boundary_mode(
    manifest: Option<&TransactionManifest>,
) -> Result<Option<String>, LakeError> {
    manifest
        .map(|manifest| {
            validated_manifest_boundary_mode(manifest.boundary_mode)
                .map(|mode| mode.status_mode().to_string())
        })
        .transpose()
}

pub(super) fn manifest_participating_partition_count(
    manifest: Option<&TransactionManifest>,
) -> Result<Option<u32>, LakeError> {
    manifest
        .map(|manifest| {
            manifest_count(
                "manifest_participating_partition_count",
                manifest.partitions.len(),
            )
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use trellara_protocol::{ManifestBoundaryMode, ManifestPartition};

    use super::*;

    fn manifest(boundary_mode: ManifestBoundaryMode) -> TransactionManifest {
        TransactionManifest {
            transaction_id: "tx-1".to_string(),
            source_commit_lsn: "0/16B6C50".to_string(),
            source_commit_timestamp_ms: 1_700_000_000_000,
            global_event_count: 2,
            partitions: vec![
                ManifestPartition {
                    id: 0,
                    event_count: 1,
                    first_total_order: 1,
                    last_total_order: 1,
                    checksum: 10,
                },
                ManifestPartition {
                    id: 1,
                    event_count: 1,
                    first_total_order: 2,
                    last_total_order: 2,
                    checksum: 20,
                },
            ],
            affected_tables: Vec::new(),
            boundary_mode: boundary_mode as i32,
        }
    }

    #[test]
    fn manifest_helpers_preserve_partitioned_scale_boundary_evidence() {
        let manifest = manifest(ManifestBoundaryMode::PartitionedScale);

        assert_eq!(manifest_id(&manifest), "tx-1:0/16B6C50:2");
        assert_eq!(
            manifest_boundary_mode(Some(&manifest)).expect("valid boundary mode"),
            Some("partitioned_scale_mode".to_string())
        );
        assert_eq!(
            manifest_participating_partition_count(Some(&manifest)).expect("valid count"),
            Some(2)
        );
    }

    #[test]
    fn manifest_helpers_fail_closed_for_missing_boundary_mode() {
        let manifest = TransactionManifest {
            boundary_mode: ManifestBoundaryMode::Unspecified as i32,
            ..manifest(ManifestBoundaryMode::PartitionedScale)
        };

        assert!(matches!(
            manifest_boundary_mode(Some(&manifest)),
            Err(LakeError::InvalidManifestBoundaryMode { boundary_mode: 0 })
        ));
    }
}
