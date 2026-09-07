use sha2::{Digest, Sha256};

#[path = "lake_epoch_digest_input.rs"]
mod lake_epoch_digest_input;
#[path = "lake_epoch_digest_manifest.rs"]
mod lake_epoch_digest_manifest;
#[path = "lake_epoch_digest_partition_skew.rs"]
mod lake_epoch_digest_partition_skew;
#[path = "lake_epoch_digest_rows.rs"]
mod lake_epoch_digest_rows;

use crate::{
    LakeEpochPartitionRollup, LakeEpochPartitionSkew, LakeEpochQuarantineEntry,
    LakeEpochSourceWatermark, LakeEpochSummary, LakeEpochTableRollup,
};
use lake_epoch_digest_manifest::render_lake_epoch_manifest;

pub(crate) use lake_epoch_digest_input::LakeEpochManifestDigestInput;

pub(crate) fn lake_epoch_manifest_digest(
    summary: &LakeEpochManifestDigestInput<'_>,
    sources: &[LakeEpochSourceWatermark],
    tables: &[LakeEpochTableRollup],
    partitions: &[LakeEpochPartitionRollup],
    partition_skew: &LakeEpochPartitionSkew,
    quarantine_entries: &[LakeEpochQuarantineEntry],
) -> String {
    let manifest = render_lake_epoch_manifest(
        summary,
        sources,
        tables,
        partitions,
        partition_skew,
        quarantine_entries,
    );
    format!("{:x}", Sha256::digest(manifest.as_bytes()))
}

pub(crate) fn lake_epoch_summary_manifest_digest(epoch: &LakeEpochSummary) -> String {
    lake_epoch_manifest_digest(
        &LakeEpochManifestDigestInput {
            epoch_id: &epoch.epoch_id,
            dataset_id: &epoch.dataset_id,
            state: epoch.state,
            straggler_policy: &epoch.straggler_policy,
            required_source_count: epoch.required_source_count,
            complete_source_count: epoch.complete_source_count,
            missing_source_count: epoch.missing_source_count,
            quarantined_source_count: epoch.quarantined_source_count,
            transaction_count: epoch.transaction_count,
            change_count: epoch.change_count,
        },
        &epoch.source_watermarks,
        &epoch.table_rollups,
        &epoch.partition_rollups,
        &epoch.partition_skew,
        &epoch.quarantine_entries,
    )
}

#[cfg(test)]
#[path = "lake_epoch_digest_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "lake_epoch_digest_partition_skew_tests.rs"]
mod partition_skew_tests;

#[cfg(test)]
#[path = "lake_epoch_digest_summary_tests.rs"]
mod summary_tests;
