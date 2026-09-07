#[path = "lake_fanin_compare.rs"]
mod lake_fanin_compare;

use std::fs;
use std::path::Path;

use crate::{
    lake_fanin_verify_recommended_next_steps, CliError, LakeEpochSummary, LakeFaninVerifyArgs,
    LakeFaninVerifyMismatchSeverity, LakeFaninVerifySummary, Result, TrellaraConfig,
};

use lake_fanin_compare::compare_lake_epochs;

impl LakeFaninVerifySummary {
    pub(crate) fn from_args(config: &TrellaraConfig, args: &LakeFaninVerifyArgs) -> Result<Self> {
        let stream_epoch = read_lake_epoch_summary(&args.stream_epoch)?;
        let lake_epoch = read_lake_epoch_summary(&args.lake_epoch)?;
        Ok(Self::from_epochs(
            config,
            args.config.as_path(),
            &args.stream_epoch,
            &args.lake_epoch,
            stream_epoch,
            lake_epoch,
            args.accept_complete_with_gaps,
        ))
    }

    pub(crate) fn from_epochs(
        config: &TrellaraConfig,
        config_path: &Path,
        stream_epoch_path: &Path,
        lake_epoch_path: &Path,
        stream_epoch: LakeEpochSummary,
        lake_epoch: LakeEpochSummary,
        accept_complete_with_gaps: bool,
    ) -> Self {
        let comparison = compare_lake_epochs(&stream_epoch, &lake_epoch, accept_complete_with_gaps);
        let recommended_next_steps = lake_fanin_verify_recommended_next_steps(
            comparison.status,
            comparison.spark_consumption_allowed,
        );
        let warning_mismatch_count = comparison
            .mismatches
            .iter()
            .filter(|mismatch| mismatch.severity == LakeFaninVerifyMismatchSeverity::Warning)
            .count();
        let blocker_mismatch_count = comparison
            .mismatches
            .iter()
            .filter(|mismatch| mismatch.severity == LakeFaninVerifyMismatchSeverity::Blocker)
            .count();
        let source_counts_match = source_counts_match(&stream_epoch, &lake_epoch);
        let checksum_rollup_match = stream_epoch.checksum_rollup == lake_epoch.checksum_rollup;

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            stream_epoch_path: stream_epoch_path.display().to_string(),
            lake_epoch_path: lake_epoch_path.display().to_string(),
            epoch_id: stream_epoch.epoch_id,
            status: comparison.status,
            stream_state: stream_epoch.state,
            lake_state: lake_epoch.state,
            stream_required_source_count: stream_epoch.required_source_count,
            stream_complete_source_count: stream_epoch.complete_source_count,
            stream_missing_source_count: stream_epoch.missing_source_count,
            stream_quarantined_source_count: stream_epoch.quarantined_source_count,
            lake_required_source_count: lake_epoch.required_source_count,
            lake_complete_source_count: lake_epoch.complete_source_count,
            lake_missing_source_count: lake_epoch.missing_source_count,
            lake_quarantined_source_count: lake_epoch.quarantined_source_count,
            source_counts_match,
            stream_checksum_rollup: stream_epoch.checksum_rollup,
            lake_checksum_rollup: lake_epoch.checksum_rollup,
            checksum_rollup_match,
            stream_customer_decision: stream_epoch.customer_decision,
            lake_customer_decision: lake_epoch.customer_decision,
            matched_check_count: comparison.compared_field_count - comparison.mismatches.len(),
            mismatch_count: comparison.mismatches.len(),
            warning_mismatch_count,
            blocker_mismatch_count,
            mismatches: comparison.mismatches,
            spark_consumption_allowed: comparison.spark_consumption_allowed,
            spark_consumption_contract: trellara_lake::LAKE_EPOCH_CONSUMER_GATE_CONTRACT,
            spark_consumption_gate: comparison.spark_consumption_gate,
            proof_command: proof_command(
                config_path,
                stream_epoch_path,
                lake_epoch_path,
                accept_complete_with_gaps,
            ),
            recommended_next_steps,
        }
    }
}

fn source_counts_match(stream_epoch: &LakeEpochSummary, lake_epoch: &LakeEpochSummary) -> bool {
    stream_epoch.required_source_count == lake_epoch.required_source_count
        && stream_epoch.complete_source_count == lake_epoch.complete_source_count
        && stream_epoch.missing_source_count == lake_epoch.missing_source_count
        && stream_epoch.quarantined_source_count == lake_epoch.quarantined_source_count
}

fn proof_command(
    config_path: &Path,
    stream_epoch_path: &Path,
    lake_epoch_path: &Path,
    accept_complete_with_gaps: bool,
) -> String {
    let mut command = format!(
        "trellara lake fanin verify --config {} --stream-epoch {} --lake-epoch {}",
        config_path.display(),
        stream_epoch_path.display(),
        lake_epoch_path.display()
    );
    if accept_complete_with_gaps {
        command.push_str(" --accept-complete-with-gaps");
    }
    command
}

pub(crate) fn read_lake_epoch_summary(path: &Path) -> Result<LakeEpochSummary> {
    let contents = fs::read_to_string(path).map_err(|source| CliError::ReadInput {
        path: path.display().to_string(),
        source,
    })?;
    Ok(serde_json::from_str(&contents)?)
}
