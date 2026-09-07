use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use trellara_lake::LakeRawCdcEpochWritePlan;

mod support;
use support::{metadata_epoch_plan, metadata_kind, require_expected_files};

use crate::epoch_metadata_validation::stable_snapshot_map;
use crate::planner::TableSeed;
use crate::planner_files::validate_completed_file_shape;
use crate::planner_table::table_plan;
use crate::{
    IcebergEpochCommitPlan, IcebergIntegrationError, IcebergMetadataTableKind,
    IcebergMetadataTableSpec, IcebergUploadedMetadataFile, Result, SNAPSHOT_PROPERTY_METADATA_KIND,
    SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergMetadataCommitBundle {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub epoch_snapshot_reference: String,
    pub raw_snapshot_ids: BTreeMap<String, i64>,
    pub supporting_metadata_commit_plan: IcebergEpochCommitPlan,
    pub completeness_commit_plan: IcebergEpochCommitPlan,
    pub visibility_rule: String,
}

pub fn plan_iceberg_metadata_commit_bundle(
    write_plan: &LakeRawCdcEpochWritePlan,
    raw_commit_plan: &IcebergEpochCommitPlan,
    specs: &[IcebergMetadataTableSpec],
    raw_snapshot_ids: BTreeMap<String, i64>,
    epoch_snapshot_reference: impl Into<String>,
    uploaded_files: Vec<IcebergUploadedMetadataFile>,
) -> Result<IcebergMetadataCommitBundle> {
    if raw_snapshot_ids.is_empty() {
        return Err(IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: raw_commit_plan
                .tables
                .iter()
                .map(|table| table.target.qualified_name())
                .collect(),
        });
    }
    let epoch_snapshot_reference = epoch_snapshot_reference.into();
    if epoch_snapshot_reference.trim().is_empty() {
        return Err(IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: Vec::new(),
        });
    }
    let by_kind = specs
        .iter()
        .map(|spec| (spec.kind, spec))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut tables = Vec::with_capacity(uploaded_files.len());
    for uploaded in uploaded_files {
        if !seen.insert(uploaded.kind) {
            return Err(IcebergIntegrationError::DuplicateCompletedDataFile {
                planned_object_key: uploaded.completed_file.planned_object_key,
            });
        }
        let spec = by_kind.get(&uploaded.kind).ok_or_else(|| {
            IcebergIntegrationError::UnplannedCompletedDataFile {
                planned_object_key: uploaded.completed_file.planned_object_key.clone(),
            }
        })?;
        validate_completed_file_shape(&uploaded.completed_file)?;
        if uploaded.completed_file.table_name != spec.lake_table_name {
            return Err(IcebergIntegrationError::CompletedDataFileMismatch {
                planned_object_key: uploaded.completed_file.planned_object_key.clone(),
                field: "metadata_table_name",
                expected: spec.lake_table_name.clone(),
                actual: uploaded.completed_file.table_name.clone(),
            });
        }
        let mut table = table_plan(
            write_plan,
            &raw_commit_plan.epoch_commit_id,
            TableSeed {
                lake_table_name: spec.lake_table_name.clone(),
                relation: uploaded.completed_file.relation.clone(),
                target: spec.target.clone(),
                data_files: vec![uploaded.completed_file],
            },
        )?;
        table.snapshot_properties.insert(
            SNAPSHOT_PROPERTY_METADATA_KIND.to_string(),
            metadata_kind(uploaded.kind).to_string(),
        );
        table.snapshot_properties.insert(
            SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS.to_string(),
            stable_snapshot_map(&raw_snapshot_ids),
        );
        tables.push((spec.commit_order, uploaded.kind, table));
    }
    require_expected_files(write_plan, &seen)?;
    tables.sort_by_key(|(order, _, _)| *order);
    let completeness_index = tables
        .iter()
        .position(|(_, kind, _)| *kind == IcebergMetadataTableKind::Completeness)
        .ok_or_else(|| IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: vec![write_plan.epoch_metadata.epochs_table.clone()],
        })?;
    let (_, _, completeness) = tables.remove(completeness_index);
    let supporting = tables
        .into_iter()
        .map(|(_, _, table)| table)
        .collect::<Vec<_>>();
    let supporting_metadata_commit_plan = metadata_epoch_plan(
        raw_commit_plan,
        supporting,
        "supporting source/table/partition/quarantine/verification metadata commits before the release marker",
    )?;
    let completeness_commit_plan = metadata_epoch_plan(
        raw_commit_plan,
        vec![completeness],
        "the _trellara_epochs completeness row commits last and is the only SQL-visible release marker",
    )?;
    Ok(IcebergMetadataCommitBundle {
        dataset_id: raw_commit_plan.dataset_id.clone(),
        epoch_id: raw_commit_plan.epoch_id.clone(),
        epoch_commit_id: raw_commit_plan.epoch_commit_id.clone(),
        epoch_snapshot_reference,
        raw_snapshot_ids,
        supporting_metadata_commit_plan,
        completeness_commit_plan,
        visibility_rule:
            "raw snapshots -> supporting metadata snapshots -> _trellara_epochs release marker"
                .to_string(),
    })
}
