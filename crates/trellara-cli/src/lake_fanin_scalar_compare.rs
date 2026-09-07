use crate::{
    lake_completeness_state_label, lake_epoch_verification_status_label, push_lake_fanin_mismatch,
    LakeEpochSummary, LakeFaninVerifyMismatch, LakeFaninVerifyMismatchSeverity,
};

pub(crate) const SCALAR_FIELD_COUNT: usize = 11;

pub(crate) fn compare_scalar_fields(
    stream_epoch: &LakeEpochSummary,
    lake_epoch: &LakeEpochSummary,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    push_lake_fanin_mismatch(
        mismatches,
        "dataset_id",
        &stream_epoch.dataset_id,
        &lake_epoch.dataset_id,
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "epoch_id",
        &stream_epoch.epoch_id,
        &lake_epoch.epoch_id,
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "state",
        lake_completeness_state_label(stream_epoch.state),
        lake_completeness_state_label(lake_epoch.state),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    compare_count_fields(stream_epoch, lake_epoch, mismatches);
    push_lake_fanin_mismatch(
        mismatches,
        "verification_status",
        lake_epoch_verification_status_label(stream_epoch.verification_status),
        lake_epoch_verification_status_label(lake_epoch.verification_status),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
}

fn compare_count_fields(
    stream_epoch: &LakeEpochSummary,
    lake_epoch: &LakeEpochSummary,
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
) {
    push_lake_fanin_mismatch(
        mismatches,
        "required_source_count",
        &stream_epoch.required_source_count.to_string(),
        &lake_epoch.required_source_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "complete_source_count",
        &stream_epoch.complete_source_count.to_string(),
        &lake_epoch.complete_source_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "missing_source_count",
        &stream_epoch.missing_source_count.to_string(),
        &lake_epoch.missing_source_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "quarantined_source_count",
        &stream_epoch.quarantined_source_count.to_string(),
        &lake_epoch.quarantined_source_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "transaction_count",
        &stream_epoch.transaction_count.to_string(),
        &lake_epoch.transaction_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "change_count",
        &stream_epoch.change_count.to_string(),
        &lake_epoch.change_count.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
    push_lake_fanin_mismatch(
        mismatches,
        "checksum_rollup",
        &stream_epoch.checksum_rollup.to_string(),
        &lake_epoch.checksum_rollup.to_string(),
        LakeFaninVerifyMismatchSeverity::Blocker,
    );
}
