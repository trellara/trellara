use std::collections::BTreeSet;

use crate::epoch_completeness::{epoch_state, verification_status};
use crate::raw_cdc::manifest::{epoch_manifest_digest, verification_id};
#[path = "metadata_quarantine.rs"]
mod metadata_quarantine;
#[path = "metadata_sources.rs"]
mod metadata_sources;
#[path = "metadata_validation.rs"]
mod metadata_validation;

use metadata_quarantine::quarantine_rows_for_sources;
use metadata_sources::{
    add_required_gap_source_rows, source_completeness_counts, validate_source_gap_evidence,
};
use metadata_validation::{validate_metadata_rollups, validate_metadata_rows};

use crate::raw_cdc_naming::lake_table_prefix;
use crate::raw_cdc_types::{
    LakeRawCdcEpochMetadataPlan, LakeRawCdcEpochPartitionRow, LakeRawCdcEpochRow,
    LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow, LakeRawCdcEpochVerificationRow,
};
use crate::{LakeEpochConfig, LakeError, LakeStragglerPolicy};

pub(crate) const RAW_CDC_VISIBILITY_RULE: &str =
    "append raw CDC files first, then publish epoch metadata only after every planned data file for the epoch is durable";

pub(crate) struct RawCdcMetadataInput {
    pub(crate) dataset_id: String,
    pub(crate) epoch_id: String,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    pub(crate) required_sources: BTreeSet<String>,
    pub(crate) straggler_policy: LakeStragglerPolicy,
    pub(crate) source_rows: Vec<LakeRawCdcEpochSourceRow>,
    pub(crate) table_rows: Vec<LakeRawCdcEpochTableRow>,
    pub(crate) partition_rows: Vec<LakeRawCdcEpochPartitionRow>,
}

pub(crate) fn build_epoch_metadata(
    mut input: RawCdcMetadataInput,
) -> Result<LakeRawCdcEpochMetadataPlan, LakeError> {
    let metadata_prefix = lake_table_prefix(&input.dataset_id, "trellara", "fanin");
    validate_metadata_rows(&input)?;
    add_required_gap_source_rows(&mut input);
    validate_source_gap_evidence(&input)?;
    validate_metadata_rollups(&input)?;
    let counts = source_completeness_counts(&input);
    let epoch_config = LakeEpochConfig::new(
        &input.epoch_id,
        &input.dataset_id,
        input.required_sources.iter().cloned(),
        input.straggler_policy.clone(),
    );
    let state = epoch_state(
        &epoch_config,
        counts.missing_source_count,
        counts.quarantined_source_count,
    );
    let manifest_digest =
        epoch_manifest_digest(&input.source_rows, &input.table_rows, &input.partition_rows);
    let quarantine_rows = quarantine_rows_for_sources(&input.epoch_id, &input.source_rows);

    Ok(LakeRawCdcEpochMetadataPlan {
        epochs_table: format!("{metadata_prefix}___trellara_epochs"),
        epoch_sources_table: format!("{metadata_prefix}___trellara_epoch_sources"),
        epoch_tables_table: format!("{metadata_prefix}___trellara_epoch_tables"),
        epoch_partitions_table: format!("{metadata_prefix}___trellara_epoch_partitions"),
        quarantine_table: format!("{metadata_prefix}___trellara_quarantine"),
        verification_table: format!("{metadata_prefix}___trellara_verification"),
        epoch_row: LakeRawCdcEpochRow {
            epoch_id: input.epoch_id.clone(),
            dataset_id: input.dataset_id.clone(),
            state,
            policy: straggler_policy_label(&input.straggler_policy).to_string(),
            opened_at: "planned_epoch_open".to_string(),
            sealed_at: "planned_after_raw_cdc_files_durable".to_string(),
            required_source_count: counts.required_source_count,
            complete_source_count: counts.complete_source_count,
            missing_source_count: counts.missing_source_count,
            quarantined_source_count: counts.quarantined_source_count,
            transaction_count: input.transaction_count,
            change_count: input.change_count,
            checksum_rollup: input.checksum_rollup,
            manifest_digest,
            iceberg_snapshot_id: None,
        },
        source_rows: input.source_rows,
        table_rows: input.table_rows,
        partition_rows: input.partition_rows,
        quarantine_rows,
        verification_row: LakeRawCdcEpochVerificationRow {
            epoch_id: input.epoch_id.clone(),
            verification_id: verification_id(
                &input.dataset_id,
                &input.epoch_id,
                input.checksum_rollup,
            ),
            input_transaction_count: input.transaction_count,
            input_change_count: input.change_count,
            lake_transaction_count: input.transaction_count,
            lake_change_count: input.change_count,
            checksum_status: verification_status(state),
            completed_at: "planned_after_raw_cdc_files_durable".to_string(),
        },
    })
}

fn straggler_policy_label(policy: &LakeStragglerPolicy) -> &'static str {
    match policy {
        LakeStragglerPolicy::WaitAllRequired => "wait_all_required",
        LakeStragglerPolicy::PublishWithGaps { .. } => "publish_with_gaps",
        LakeStragglerPolicy::QuarantineOnGap => "quarantine_on_gap",
    }
}
