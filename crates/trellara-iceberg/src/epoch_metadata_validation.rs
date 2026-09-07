use std::collections::BTreeMap;

use trellara_lake::{
    raw_cdc_epoch_metadata_object_key_hint, LakeEpochConsumerOptions,
    LakeRawCdcEpochMetadataParquetWriteEvidence, LakeRawCdcEpochWritePlan,
};

use crate::planner_mappings::validated_mappings;
use crate::{
    IcebergCommitConfig, IcebergEpochCommitPlan, IcebergIntegrationError, IcebergTableIdentifier,
    Result,
};

pub(crate) fn metadata_target(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<IcebergTableIdentifier> {
    validated_mappings(config)?
        .get(&write_plan.epoch_metadata.epochs_table)
        .cloned()
        .ok_or_else(|| IcebergIntegrationError::MissingTableMapping {
            lake_table_name: write_plan.epoch_metadata.epochs_table.clone(),
        })
}

pub(crate) fn validate_raw_commit_plan_boundary(
    write_plan: &LakeRawCdcEpochWritePlan,
    raw_commit_plan: &IcebergEpochCommitPlan,
) -> Result<()> {
    compare(
        "dataset_id",
        &write_plan.dataset_id,
        &raw_commit_plan.dataset_id,
    )?;
    compare("epoch_id", &write_plan.epoch_id, &raw_commit_plan.epoch_id)?;
    compare(
        "manifest_digest",
        &write_plan.epoch_metadata.epoch_row.manifest_digest,
        &raw_commit_plan.manifest_digest,
    )?;
    compare(
        "checksum_rollup",
        &write_plan.checksum_rollup.to_string(),
        &raw_commit_plan.checksum_rollup.to_string(),
    )?;
    compare(
        "transaction_count",
        &write_plan.transaction_count.to_string(),
        &raw_commit_plan.transaction_count.to_string(),
    )?;
    compare(
        "change_count",
        &write_plan.change_count.to_string(),
        &raw_commit_plan.change_count.to_string(),
    )
}

pub(crate) fn validate_metadata_file(
    write_plan: &LakeRawCdcEpochWritePlan,
    raw_snapshot_ids: &BTreeMap<String, i64>,
    epoch_snapshot_reference: &str,
    metadata_file: &LakeRawCdcEpochMetadataParquetWriteEvidence,
) -> Result<()> {
    let expected_key = raw_cdc_epoch_metadata_object_key_hint(
        &write_plan.epoch_id,
        &write_plan.epoch_metadata.epochs_table,
    );
    compare(
        "metadata.planned_object_key",
        &expected_key,
        &metadata_file.planned_object_key,
    )?;
    compare(
        "metadata.table_name",
        &write_plan.epoch_metadata.epochs_table,
        &metadata_file.table_name,
    )?;
    compare(
        "metadata.dataset_id",
        &write_plan.dataset_id,
        &metadata_file.dataset_id,
    )?;
    compare(
        "metadata.epoch_id",
        &write_plan.epoch_id,
        &metadata_file.epoch_id,
    )?;
    compare(
        "metadata.manifest_digest",
        &write_plan.epoch_metadata.epoch_row.manifest_digest,
        &metadata_file.manifest_digest,
    )?;
    compare(
        "metadata.checksum_rollup",
        &write_plan.checksum_rollup.to_string(),
        &metadata_file.checksum_rollup.to_string(),
    )?;
    compare(
        "metadata.record_count",
        "1",
        &metadata_file.record_count.to_string(),
    )?;
    compare(
        "metadata.iceberg_snapshot_id",
        epoch_snapshot_reference,
        &metadata_file.iceberg_snapshot_id,
    )?;
    validate_non_empty_snapshot_map(
        "metadata.raw_table_snapshot_ids",
        metadata_file.planned_object_key.clone(),
        &metadata_file.raw_table_snapshot_ids,
    )?;
    if &metadata_file.raw_table_snapshot_ids != raw_snapshot_ids {
        return Err(IcebergIntegrationError::CompletedDataFileMismatch {
            planned_object_key: metadata_file.planned_object_key.clone(),
            field: "raw_table_snapshot_ids",
            expected: stable_snapshot_map(raw_snapshot_ids),
            actual: stable_snapshot_map(&metadata_file.raw_table_snapshot_ids),
        });
    }
    Ok(())
}

pub(crate) fn consumer_options(config: &IcebergCommitConfig) -> LakeEpochConsumerOptions {
    if config.accept_complete_with_gaps {
        LakeEpochConsumerOptions::accepting_complete_with_gaps()
    } else {
        LakeEpochConsumerOptions::strict()
    }
}

pub(crate) fn stable_snapshot_map(raw_snapshot_ids: &BTreeMap<String, i64>) -> String {
    raw_snapshot_ids
        .iter()
        .map(|(target, snapshot_id)| format!("{target}={snapshot_id}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn validate_non_empty_snapshot_map(
    field: &'static str,
    planned_object_key: String,
    raw_snapshot_ids: &BTreeMap<String, i64>,
) -> Result<()> {
    if !raw_snapshot_ids.is_empty() {
        return Ok(());
    }
    Err(IcebergIntegrationError::InvalidCompletedDataFile {
        planned_object_key,
        field,
        reason:
            "must include every raw changelog table snapshot before _trellara_epochs is written"
                .to_string(),
    })
}

fn compare(field: &'static str, expected: &str, actual: &str) -> Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(IcebergIntegrationError::WritePlanBoundaryMismatch {
            field,
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}
