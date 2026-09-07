use trellara_lake::{
    ensure_lake_epoch_consumable, LakeRawCdcEpochMetadataParquetWriteEvidence,
    LakeRawCdcEpochWritePlan,
};

use crate::epoch_metadata_schema::{
    epoch_metadata_columns, epoch_metadata_partition_fields, epoch_metadata_schema_fingerprint,
};
use crate::epoch_metadata_validation::{
    consumer_options, metadata_target, stable_snapshot_map, validate_metadata_file,
    validate_raw_commit_plan_boundary,
};
use crate::planner::TableSeed;
use crate::planner_files::validate_completed_file_shape;
use crate::planner_table::table_plan;
use crate::{
    iceberg_epoch_commit_summary, IcebergCommitConfig, IcebergCompletedDataFile,
    IcebergEpochCommitPlan, IcebergEpochMetadataAppendPlan, IcebergEpochMetadataTableSpec,
    IcebergIntegrationError, IcebergTableCommitReceipt, Result, ICEBERG_EPOCH_METADATA_KIND,
    ICEBERG_EPOCH_METADATA_RELATION, SNAPSHOT_PROPERTY_METADATA_KIND,
    SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS,
};

pub fn plan_iceberg_epoch_metadata_table_spec(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<IcebergEpochMetadataTableSpec> {
    let target = metadata_target(write_plan, config)?;
    let columns = epoch_metadata_columns();
    let partition_fields = epoch_metadata_partition_fields();
    Ok(IcebergEpochMetadataTableSpec {
        lake_table_name: write_plan.epoch_metadata.epochs_table.clone(),
        relation: ICEBERG_EPOCH_METADATA_RELATION.to_string(),
        target,
        schema_fingerprint_sha256: epoch_metadata_schema_fingerprint(&columns, &partition_fields),
        columns,
        partition_fields,
        write_mode: "append_only_epoch_completeness".to_string(),
        visibility_rule:
            "append one _trellara_epochs row only after every raw changelog table receipt validates"
                .to_string(),
    })
}

pub fn plan_iceberg_epoch_metadata_append(
    write_plan: &LakeRawCdcEpochWritePlan,
    raw_commit_plan: &IcebergEpochCommitPlan,
    config: &IcebergCommitConfig,
    raw_receipts: &[IcebergTableCommitReceipt],
    metadata_file: LakeRawCdcEpochMetadataParquetWriteEvidence,
) -> Result<IcebergEpochMetadataAppendPlan> {
    validate_raw_commit_plan_boundary(write_plan, raw_commit_plan)?;
    ensure_lake_epoch_consumable(
        &write_plan.epoch_metadata.epoch_row,
        &write_plan.epoch_metadata.verification_row,
        consumer_options(config),
    )?;
    let summary = iceberg_epoch_commit_summary(raw_commit_plan, raw_receipts)?;
    if !summary.ready_for_epoch_metadata {
        return Err(IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: summary.missing_tables,
        });
    }
    let epoch_snapshot_reference = summary.epoch_snapshot_reference.ok_or_else(|| {
        IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: Vec::new(),
        }
    })?;
    validate_metadata_file(
        write_plan,
        &summary.snapshot_ids,
        &epoch_snapshot_reference,
        &metadata_file,
    )?;

    let append_plan = metadata_table_append_plan(
        write_plan,
        raw_commit_plan,
        config,
        metadata_file,
        &summary.snapshot_ids,
    )?;
    let metadata_table = append_plan.target.qualified_name();
    let metadata_commit_plan =
        metadata_commit_plan(raw_commit_plan, append_plan, raw_commit_plan.file_count);

    Ok(IcebergEpochMetadataAppendPlan {
        dataset_id: raw_commit_plan.dataset_id.clone(),
        epoch_id: raw_commit_plan.epoch_id.clone(),
        epoch_commit_id: raw_commit_plan.epoch_commit_id.clone(),
        epoch_snapshot_reference,
        raw_snapshot_ids: summary.snapshot_ids,
        metadata_table,
        metadata_commit_plan,
    })
}

fn metadata_table_append_plan(
    write_plan: &LakeRawCdcEpochWritePlan,
    raw_commit_plan: &IcebergEpochCommitPlan,
    config: &IcebergCommitConfig,
    metadata_file: LakeRawCdcEpochMetadataParquetWriteEvidence,
    raw_snapshot_ids: &std::collections::BTreeMap<String, i64>,
) -> Result<crate::IcebergTableAppendPlan> {
    let completed_file = IcebergCompletedDataFile::try_from(metadata_file)?;
    validate_completed_file_shape(&completed_file)?;
    let mut append_plan = table_plan(
        write_plan,
        &raw_commit_plan.epoch_commit_id,
        TableSeed {
            lake_table_name: write_plan.epoch_metadata.epochs_table.clone(),
            relation: ICEBERG_EPOCH_METADATA_RELATION.to_string(),
            target: metadata_target(write_plan, config)?,
            data_files: vec![completed_file],
        },
    )?;
    append_plan.snapshot_properties.insert(
        SNAPSHOT_PROPERTY_METADATA_KIND.to_string(),
        ICEBERG_EPOCH_METADATA_KIND.to_string(),
    );
    append_plan.snapshot_properties.insert(
        SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS.to_string(),
        stable_snapshot_map(raw_snapshot_ids),
    );
    Ok(append_plan)
}

fn metadata_commit_plan(
    raw_commit_plan: &IcebergEpochCommitPlan,
    append_plan: crate::IcebergTableAppendPlan,
    raw_file_count: usize,
) -> IcebergEpochCommitPlan {
    IcebergEpochCommitPlan {
        dataset_id: raw_commit_plan.dataset_id.clone(),
        epoch_id: raw_commit_plan.epoch_id.clone(),
        epoch_commit_id: raw_commit_plan.epoch_commit_id.clone(),
        manifest_digest: raw_commit_plan.manifest_digest.clone(),
        checksum_rollup: raw_commit_plan.checksum_rollup,
        transaction_count: raw_commit_plan.transaction_count,
        change_count: raw_commit_plan.change_count,
        file_count: 1,
        record_count: 1,
        accepted_complete_with_gaps: raw_commit_plan.accepted_complete_with_gaps,
        epoch_visibility_rule: format!(
            "the _trellara_epochs row is the SQL-visible release marker after {raw_file_count} raw changelog files prove completeness"
        ),
        tables: vec![append_plan],
    }
}
