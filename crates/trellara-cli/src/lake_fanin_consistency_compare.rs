use crate::{
    lake_epoch_partition_skew, lake_epoch_summary_manifest_digest, LakeEpochSummary,
    LakeFaninVerifyMismatch, LakeFaninVerifyMismatchSeverity,
};

use super::lake_fanin_partition_consistency::validate_partition_rollup_rows;
use super::lake_fanin_quarantine_consistency::validate_quarantine_entry_rows;
use super::lake_fanin_source_consistency::validate_source_watermark_rows;
use super::lake_fanin_source_rollup_consistency::{
    validate_source_count_rollup, validate_source_watermark_cardinality,
    validate_source_watermark_rollup,
};
use super::lake_fanin_state_consistency::validate_epoch_state_consistency;
use super::lake_fanin_table_consistency::validate_table_rollup_rows;

pub(crate) const INTERNAL_CONSISTENCY_FIELD_COUNT: usize = 8;

pub(crate) fn compare_internal_consistency(
    stream_epoch: &LakeEpochSummary,
    lake_epoch: &LakeEpochSummary,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    push_stream_consistency_mismatch(validate_epoch(stream_epoch), mismatches);
    push_lake_consistency_mismatch(validate_epoch(lake_epoch), mismatches);
}

fn push_stream_consistency_mismatch(
    result: Result<(), String>,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    if let Err(error) = result {
        mismatches.push(LakeFaninVerifyMismatch {
            field: "stream_epoch_consistency".to_string(),
            stream_value: error,
            lake_value: "valid".to_string(),
            severity: LakeFaninVerifyMismatchSeverity::Blocker,
        });
    }
}

fn push_lake_consistency_mismatch(
    result: Result<(), String>,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    if let Err(error) = result {
        mismatches.push(LakeFaninVerifyMismatch {
            field: "lake_epoch_consistency".to_string(),
            stream_value: "valid".to_string(),
            lake_value: error,
            severity: LakeFaninVerifyMismatchSeverity::Blocker,
        });
    }
}

fn validate_epoch(epoch: &LakeEpochSummary) -> Result<(), String> {
    validate_source_watermark_rows(epoch)?;
    validate_table_rollup_rows(epoch)?;
    validate_partition_rollup_rows(epoch)?;
    validate_partition_skew(epoch)?;
    validate_source_count_rollup(epoch)?;
    validate_source_watermark_rollup(epoch)?;
    validate_source_watermark_cardinality(epoch)?;
    validate_quarantine_entries(epoch)?;
    validate_quarantine_entry_rows(epoch)?;
    validate_epoch_state_consistency(epoch)?;
    validate_manifest_digest(epoch)
}

fn validate_partition_skew(epoch: &LakeEpochSummary) -> Result<(), String> {
    let expected = lake_epoch_partition_skew(&epoch.partition_rollups);
    if epoch.partition_skew == expected {
        Ok(())
    } else {
        Err("stored partition_skew does not match partition_rollups".to_string())
    }
}

fn validate_manifest_digest(epoch: &LakeEpochSummary) -> Result<(), String> {
    let expected = lake_epoch_summary_manifest_digest(epoch);

    if epoch.manifest_digest == expected {
        Ok(())
    } else {
        Err(format!(
            "manifest_digest does not match epoch rows: stored={} recomputed={expected}",
            epoch.manifest_digest
        ))
    }
}

fn validate_quarantine_entries(epoch: &LakeEpochSummary) -> Result<(), String> {
    match (
        epoch.quarantined_source_count,
        epoch.quarantine_entries.is_empty(),
    ) {
        (0, true) => Ok(()),
        (0, false) => Err("quarantine entries present without quarantined sources".to_string()),
        (_, false) => Ok(()),
        (_, true) => Err("quarantined sources require quarantine entries".to_string()),
    }
}
