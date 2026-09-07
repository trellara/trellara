use crate::lake_fanin_run_types::{LakeFaninRunStatus, LakeFaninRunSummary};
use crate::TrellaraConfig;

impl LakeFaninRunSummary {
    pub(crate) fn from_writer_plan(
        config: &TrellaraConfig,
        plan: trellara_lake::LakeRawCdcEpochWritePlan,
    ) -> Self {
        let trial_status = trial_status(&plan);
        let replay_safe = plan.skipped_dataset_transaction_count == 0;
        let spark_release_gate = spark_release_gate(&plan);

        Self {
            dataset_id: plan.dataset_id.clone(),
            epoch_id: plan.epoch_id.clone(),
            mode: config.dataset.mode.to_string(),
            trial_status,
            transaction_count: plan.transaction_count,
            change_count: plan.change_count,
            duplicate_replay_count: plan.duplicate_transaction_count,
            skipped_dataset_transaction_count: plan.skipped_dataset_transaction_count,
            data_file_count: plan.data_file_count,
            source_bucket_count: plan.committer_topology.source_bucket_count,
            replay_safe,
            spark_release_gate,
            source_ack_boundary: plan.committer_topology.source_ack_boundary.clone(),
            catalog_backpressure_rule: plan.committer_topology.catalog_backpressure_rule.clone(),
            bounded_trial_note:
                "dry-run local trial: no Iceberg catalog writes are performed; object keys and metadata rows are planned for operator review"
                    .to_string(),
            writer_plan: plan,
        }
    }
}

fn trial_status(plan: &trellara_lake::LakeRawCdcEpochWritePlan) -> LakeFaninRunStatus {
    if plan.data_file_count == 0 {
        LakeFaninRunStatus::BlockedNoDataFiles
    } else if !writer_epoch_is_consumable(plan) {
        LakeFaninRunStatus::BlockedEpochNotConsumable
    } else if plan.duplicate_transaction_count > 0 {
        LakeFaninRunStatus::PlannedWithDuplicateReplays
    } else {
        LakeFaninRunStatus::PlannedDryRun
    }
}

fn spark_release_gate(plan: &trellara_lake::LakeRawCdcEpochWritePlan) -> String {
    if writer_epoch_is_consumable(plan) {
        "publish Spark jobs only after trellara lake fanin verify reports match for this epoch"
            .to_string()
    } else {
        format!(
            "blocked: writer plan epoch state {} with verification {} is not consumable",
            completeness_state_label(plan.epoch_metadata.epoch_row.state),
            verification_status_label(plan.epoch_metadata.verification_row.checksum_status)
        )
    }
}

fn writer_epoch_is_consumable(plan: &trellara_lake::LakeRawCdcEpochWritePlan) -> bool {
    trellara_lake::ensure_lake_epoch_consumable(
        &plan.epoch_metadata.epoch_row,
        &plan.epoch_metadata.verification_row,
        trellara_lake::LakeEpochConsumerOptions::strict(),
    )
    .is_ok()
}

fn completeness_state_label(state: trellara_lake::LakeCompletenessState) -> &'static str {
    match state {
        trellara_lake::LakeCompletenessState::Open => "open",
        trellara_lake::LakeCompletenessState::Sealing => "sealing",
        trellara_lake::LakeCompletenessState::Complete => "complete",
        trellara_lake::LakeCompletenessState::CompleteWithGaps => "complete_with_gaps",
        trellara_lake::LakeCompletenessState::Quarantined => "quarantined",
        trellara_lake::LakeCompletenessState::Reseeding => "reseeding",
        trellara_lake::LakeCompletenessState::FailedRecoverable => "failed_recoverable",
    }
}

fn verification_status_label(status: trellara_lake::LakeEpochVerificationStatus) -> &'static str {
    match status {
        trellara_lake::LakeEpochVerificationStatus::Match => "match",
        trellara_lake::LakeEpochVerificationStatus::Mismatch => "mismatch",
        trellara_lake::LakeEpochVerificationStatus::Unknown => "unknown",
    }
}
