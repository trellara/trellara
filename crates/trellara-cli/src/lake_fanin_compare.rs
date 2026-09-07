#[path = "lake_fanin_consistency_compare.rs"]
mod lake_fanin_consistency_compare;
#[path = "lake_fanin_consumer_gate.rs"]
mod lake_fanin_consumer_gate;
#[path = "lake_fanin_fingerprints.rs"]
mod lake_fanin_fingerprints;
#[path = "lake_fanin_partition_consistency.rs"]
mod lake_fanin_partition_consistency;
#[path = "lake_fanin_quarantine_consistency.rs"]
mod lake_fanin_quarantine_consistency;
#[path = "lake_fanin_recovery_gate.rs"]
mod lake_fanin_recovery_gate;
#[path = "lake_fanin_scalar_compare.rs"]
mod lake_fanin_scalar_compare;
#[path = "lake_fanin_semantic_compare.rs"]
mod lake_fanin_semantic_compare;
#[path = "lake_fanin_source_consistency.rs"]
mod lake_fanin_source_consistency;
#[path = "lake_fanin_source_rollup_consistency.rs"]
mod lake_fanin_source_rollup_consistency;
#[path = "lake_fanin_spark_gate.rs"]
mod lake_fanin_spark_gate;
#[path = "lake_fanin_state_consistency.rs"]
mod lake_fanin_state_consistency;
#[path = "lake_fanin_table_consistency.rs"]
mod lake_fanin_table_consistency;

use crate::{
    LakeEpochSummary, LakeFaninVerifyMismatch, LakeFaninVerifyMismatchSeverity,
    LakeFaninVerifyStatus,
};
use lake_fanin_consistency_compare::{
    compare_internal_consistency, INTERNAL_CONSISTENCY_FIELD_COUNT,
};
use lake_fanin_scalar_compare::{compare_scalar_fields, SCALAR_FIELD_COUNT};
use lake_fanin_semantic_compare::{compare_semantic_fields, SEMANTIC_FIELD_COUNT};
use lake_fanin_spark_gate::{spark_consumption_allowed, spark_consumption_gate};

pub(crate) struct LakeFaninComparison {
    pub(crate) mismatches: Vec<LakeFaninVerifyMismatch>,
    pub(crate) status: LakeFaninVerifyStatus,
    pub(crate) spark_consumption_allowed: bool,
    pub(crate) spark_consumption_gate: String,
    pub(crate) compared_field_count: usize,
}

pub(crate) fn compare_lake_epochs(
    stream_epoch: &LakeEpochSummary,
    lake_epoch: &LakeEpochSummary,
    accept_complete_with_gaps: bool,
) -> LakeFaninComparison {
    let mut mismatches = Vec::new();

    compare_scalar_fields(stream_epoch, lake_epoch, &mut mismatches);
    compare_semantic_fields(stream_epoch, lake_epoch, &mut mismatches);
    compare_internal_consistency(stream_epoch, lake_epoch, &mut mismatches);

    let status = status_for_mismatches(&mismatches);
    LakeFaninComparison {
        mismatches,
        status,
        spark_consumption_allowed: spark_consumption_allowed(
            status,
            lake_epoch,
            accept_complete_with_gaps,
        ),
        spark_consumption_gate: spark_consumption_gate(
            status,
            lake_epoch,
            accept_complete_with_gaps,
        ),
        compared_field_count: SCALAR_FIELD_COUNT
            + SEMANTIC_FIELD_COUNT
            + INTERNAL_CONSISTENCY_FIELD_COUNT,
    }
}

fn status_for_mismatches(mismatches: &[LakeFaninVerifyMismatch]) -> LakeFaninVerifyStatus {
    if mismatches
        .iter()
        .any(|mismatch| mismatch.severity == LakeFaninVerifyMismatchSeverity::Blocker)
    {
        LakeFaninVerifyStatus::Blocked
    } else if mismatches.is_empty() {
        LakeFaninVerifyStatus::Match
    } else {
        LakeFaninVerifyStatus::Mismatch
    }
}
