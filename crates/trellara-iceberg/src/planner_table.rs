use std::collections::BTreeMap;

use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::planner::TableSeed;
use crate::planner_digest::{table_commit_id, uuid_from_hex_digest};
use crate::{
    IcebergCommitRequirements, IcebergIntegrationError, IcebergTableAppendPlan, Result,
    SNAPSHOT_PROPERTY_CHANGE_COUNT, SNAPSHOT_PROPERTY_CHECKSUM_ROLLUP,
    SNAPSHOT_PROPERTY_DATASET_ID, SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID, SNAPSHOT_PROPERTY_EPOCH_ID,
    SNAPSHOT_PROPERTY_FILE_COUNT, SNAPSHOT_PROPERTY_MANIFEST_DIGEST,
    SNAPSHOT_PROPERTY_RECORD_COUNT, SNAPSHOT_PROPERTY_TABLE_COMMIT_ID,
    SNAPSHOT_PROPERTY_TRANSACTION_COUNT,
};

pub(crate) fn table_plan(
    write_plan: &LakeRawCdcEpochWritePlan,
    epoch_commit_id: &str,
    table: TableSeed,
) -> Result<IcebergTableAppendPlan> {
    let record_count = table
        .data_files
        .iter()
        .try_fold(0u64, |count, file| count.checked_add(file.record_count))
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "table.record_count",
        })?;
    let table_commit_id = table_commit_id(epoch_commit_id, &table);
    let commit_uuid = uuid_from_hex_digest(&table_commit_id);
    let file_count = table.data_files.len();
    let snapshot_properties = snapshot_properties(
        write_plan,
        epoch_commit_id,
        &table_commit_id,
        file_count,
        record_count,
    );

    Ok(IcebergTableAppendPlan {
        lake_table_name: table.lake_table_name,
        relation: table.relation,
        target: table.target,
        epoch_commit_id: epoch_commit_id.to_string(),
        table_commit_id,
        commit_uuid,
        file_count,
        record_count,
        requirements: IcebergCommitRequirements::default(),
        snapshot_properties,
        data_files: table.data_files,
    })
}

fn snapshot_properties(
    write_plan: &LakeRawCdcEpochWritePlan,
    epoch_commit_id: &str,
    table_commit_id: &str,
    file_count: usize,
    record_count: u64,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID.to_string(),
            epoch_commit_id.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_TABLE_COMMIT_ID.to_string(),
            table_commit_id.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_DATASET_ID.to_string(),
            write_plan.dataset_id.clone(),
        ),
        (
            SNAPSHOT_PROPERTY_EPOCH_ID.to_string(),
            write_plan.epoch_id.clone(),
        ),
        (
            SNAPSHOT_PROPERTY_MANIFEST_DIGEST.to_string(),
            write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
        ),
        (
            SNAPSHOT_PROPERTY_CHECKSUM_ROLLUP.to_string(),
            write_plan.checksum_rollup.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_TRANSACTION_COUNT.to_string(),
            write_plan.transaction_count.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_CHANGE_COUNT.to_string(),
            write_plan.change_count.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_FILE_COUNT.to_string(),
            file_count.to_string(),
        ),
        (
            SNAPSHOT_PROPERTY_RECORD_COUNT.to_string(),
            record_count.to_string(),
        ),
    ])
}
