use crate::{
    push_lake_fanin_mismatch, LakeEpochSummary, LakeFaninVerifyMismatch,
    LakeFaninVerifyMismatchSeverity,
};

use super::lake_fanin_fingerprints::{
    lake_epoch_partition_rollup_fingerprint, lake_epoch_quarantine_fingerprint,
    lake_epoch_source_watermark_fingerprint, lake_epoch_table_rollup_fingerprint,
    lake_epoch_watermark_rollup_fingerprint,
};

pub(crate) const SEMANTIC_FIELD_COUNT: usize = 9;

pub(crate) fn compare_semantic_fields(
    stream_epoch: &LakeEpochSummary,
    lake_epoch: &LakeEpochSummary,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    push_lake_fanin_mismatch(
        mismatches,
        "customer_decision",
        &stream_epoch.customer_decision,
        &lake_epoch.customer_decision,
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "duplicate_replay_count",
        &stream_epoch.duplicate_replay_count.to_string(),
        &lake_epoch.duplicate_replay_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Warning,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "straggler_policy",
        &stream_epoch.straggler_policy,
        &lake_epoch.straggler_policy,
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "table_rollups",
        &lake_epoch_table_rollup_fingerprint(stream_epoch),
        &lake_epoch_table_rollup_fingerprint(lake_epoch),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "source_watermarks",
        &lake_epoch_source_watermark_fingerprint(stream_epoch),
        &lake_epoch_source_watermark_fingerprint(lake_epoch),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "partition_rollups",
        &lake_epoch_partition_rollup_fingerprint(stream_epoch),
        &lake_epoch_partition_rollup_fingerprint(lake_epoch),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "watermark_rollup",
        &lake_epoch_watermark_rollup_fingerprint(stream_epoch),
        &lake_epoch_watermark_rollup_fingerprint(lake_epoch),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "manifest_digest",
        &stream_epoch.manifest_digest,
        &lake_epoch.manifest_digest,
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "quarantine_entries",
        &lake_epoch_quarantine_fingerprint(stream_epoch),
        &lake_epoch_quarantine_fingerprint(lake_epoch),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
}
