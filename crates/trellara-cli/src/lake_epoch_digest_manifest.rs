use crate::{
    LakeEpochPartitionRollup, LakeEpochPartitionSkew, LakeEpochQuarantineEntry,
    LakeEpochSourceWatermark, LakeEpochTableRollup,
};

use super::lake_epoch_digest_input::{completeness_state_label, LakeEpochManifestDigestInput};
use super::lake_epoch_digest_partition_skew::push_partition_skew_lines;
use super::lake_epoch_digest_rows::{
    push_partition_lines, push_quarantine_lines, push_source_lines, push_table_lines,
};

pub(super) fn render_lake_epoch_manifest(
    summary: &LakeEpochManifestDigestInput<'_>,
    sources: &[LakeEpochSourceWatermark],
    tables: &[LakeEpochTableRollup],
    partitions: &[LakeEpochPartitionRollup],
    partition_skew: &LakeEpochPartitionSkew,
    quarantine_entries: &[LakeEpochQuarantineEntry],
) -> String {
    let mut manifest = String::new();
    manifest.push_str("trellara_lake_epoch_manifest_v1\n");
    push_summary_lines(&mut manifest, summary);
    push_source_lines(&mut manifest, sources);
    push_table_lines(&mut manifest, tables);
    push_partition_lines(&mut manifest, partitions);
    push_partition_skew_lines(&mut manifest, partition_skew);
    push_quarantine_lines(&mut manifest, quarantine_entries);
    manifest
}

fn push_summary_lines(manifest: &mut String, summary: &LakeEpochManifestDigestInput<'_>) {
    push_manifest_line(manifest, "epoch_id", summary.epoch_id);
    push_manifest_line(manifest, "dataset_id", summary.dataset_id);
    push_manifest_line(manifest, "state", completeness_state_label(summary.state));
    push_manifest_line(manifest, "straggler_policy", summary.straggler_policy);
    push_manifest_line(
        manifest,
        "required_source_count",
        &summary.required_source_count.to_string(),
    );
    push_manifest_line(
        manifest,
        "complete_source_count",
        &summary.complete_source_count.to_string(),
    );
    push_manifest_line(
        manifest,
        "missing_source_count",
        &summary.missing_source_count.to_string(),
    );
    push_manifest_line(
        manifest,
        "quarantined_source_count",
        &summary.quarantined_source_count.to_string(),
    );
    push_manifest_line(
        manifest,
        "transaction_count",
        &summary.transaction_count.to_string(),
    );
    push_manifest_line(manifest, "change_count", &summary.change_count.to_string());
}

pub(super) fn push_manifest_line(manifest: &mut String, key: &str, value: &str) {
    manifest.push_str(key);
    manifest.push('=');
    manifest.push_str(value);
    manifest.push('\n');
}
