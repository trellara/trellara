use crate::{TransactionInspectManifest, TransactionInspectPartition};

impl TransactionInspectManifest {
    pub(crate) fn from_manifest(manifest: &trellara_protocol::TransactionManifest) -> Self {
        let boundary_mode =
            trellara_protocol::ManifestBoundaryMode::try_from(manifest.boundary_mode)
                .unwrap_or(trellara_protocol::ManifestBoundaryMode::Unspecified)
                .status_mode()
                .to_string();
        Self {
            boundary_mode,
            global_event_count: manifest.global_event_count,
            participating_partition_count: manifest.partitions.len(),
            checksum: manifest.compute_checksum(),
            source_commit_lsn: manifest.source_commit_lsn.clone(),
            source_commit_timestamp_ms: manifest.source_commit_timestamp_ms,
            partitions: manifest
                .partitions
                .iter()
                .map(|partition| TransactionInspectPartition {
                    id: partition.id,
                    event_count: partition.event_count,
                    first_total_order: partition.first_total_order,
                    last_total_order: partition.last_total_order,
                    checksum: partition.checksum,
                })
                .collect(),
        }
    }
}
