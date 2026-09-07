use std::collections::BTreeMap;

use trellara_lake::{
    ensure_lake_epoch_consumable, LakeEpochConsumerOptions, LakeRawCdcEpochWritePlan,
};

use crate::planner_boundary::validate_write_plan_boundary;
use crate::planner_digest::epoch_commit_id;
use crate::planner_files::{completed_files_by_key, validate_completed_file};
use crate::planner_mappings::validated_mappings;
use crate::planner_table::table_plan;
use crate::{
    IcebergCommitConfig, IcebergCompletedDataFile, IcebergEpochCommitPlan, IcebergIntegrationError,
    IcebergTableIdentifier, Result,
};

pub fn plan_iceberg_epoch_commit(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
    completed_files: Vec<IcebergCompletedDataFile>,
) -> Result<IcebergEpochCommitPlan> {
    validate_write_plan_boundary(write_plan)?;
    let consumer_options = if config.accept_complete_with_gaps {
        LakeEpochConsumerOptions::accepting_complete_with_gaps()
    } else {
        LakeEpochConsumerOptions::strict()
    };
    let consumer_decision = ensure_lake_epoch_consumable(
        &write_plan.epoch_metadata.epoch_row,
        &write_plan.epoch_metadata.verification_row,
        consumer_options,
    )?;
    let mappings = validated_mappings(config)?;
    let mut completed = completed_files_by_key(completed_files)?;
    let mut tables = BTreeMap::<String, TableSeed>::new();

    for planned_file in &write_plan.data_files {
        let completed_file = completed
            .remove(&planned_file.object_key_hint)
            .ok_or_else(|| IcebergIntegrationError::MissingCompletedDataFile {
                planned_object_key: planned_file.object_key_hint.clone(),
            })?;
        validate_completed_file(planned_file, &completed_file)?;
        let target = mappings.get(&planned_file.table_name).ok_or_else(|| {
            IcebergIntegrationError::MissingTableMapping {
                lake_table_name: planned_file.table_name.clone(),
            }
        })?;
        let table = tables
            .entry(planned_file.table_name.clone())
            .or_insert_with(|| TableSeed {
                lake_table_name: planned_file.table_name.clone(),
                relation: planned_file.relation.clone(),
                target: target.clone(),
                data_files: Vec::new(),
            });
        if table.relation != planned_file.relation {
            return Err(IcebergIntegrationError::WritePlanBoundaryMismatch {
                field: "table_relation",
                expected: table.relation.clone(),
                actual: planned_file.relation.clone(),
            });
        }
        table.data_files.push(completed_file);
    }

    if let Some((planned_object_key, _)) = completed.into_iter().next() {
        return Err(IcebergIntegrationError::UnplannedCompletedDataFile { planned_object_key });
    }

    let file_count = tables
        .values()
        .try_fold(0usize, |count, table| {
            count.checked_add(table.data_files.len())
        })
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "file_count",
        })?;
    let record_count = tables
        .values()
        .try_fold(0u64, |count, table| {
            table
                .data_files
                .iter()
                .try_fold(count, |table_count, file| {
                    table_count.checked_add(file.record_count)
                })
        })
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "record_count",
        })?;
    let expected_record_count = u64::try_from(write_plan.change_count).map_err(|_| {
        IcebergIntegrationError::CountOverflow {
            field: "change_count",
        }
    })?;
    if record_count != expected_record_count {
        return Err(IcebergIntegrationError::WritePlanBoundaryMismatch {
            field: "record_count",
            expected: expected_record_count.to_string(),
            actual: record_count.to_string(),
        });
    }

    let epoch_commit_id = epoch_commit_id(write_plan, tables.values());
    let table_plans = tables
        .into_values()
        .map(|mut table| {
            table
                .data_files
                .sort_by(|left, right| left.planned_object_key.cmp(&right.planned_object_key));
            table_plan(write_plan, &epoch_commit_id, table)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(IcebergEpochCommitPlan {
        dataset_id: write_plan.dataset_id.clone(),
        epoch_id: write_plan.epoch_id.clone(),
        epoch_commit_id,
        manifest_digest: write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
        checksum_rollup: write_plan.checksum_rollup,
        transaction_count: write_plan.transaction_count,
        change_count: write_plan.change_count,
        file_count,
        record_count,
        accepted_complete_with_gaps: consumer_decision.accepted_gaps,
        epoch_visibility_rule:
            "publish Trellara epoch metadata only after every Iceberg table append receipt matches this epoch commit"
                .to_string(),
        tables: table_plans,
    })
}

#[derive(Clone)]
pub(crate) struct TableSeed {
    pub(crate) lake_table_name: String,
    pub(crate) relation: String,
    pub(crate) target: IcebergTableIdentifier,
    pub(crate) data_files: Vec<IcebergCompletedDataFile>,
}
