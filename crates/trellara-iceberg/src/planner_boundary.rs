use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::{IcebergIntegrationError, Result};

pub(crate) fn validate_write_plan_boundary(write_plan: &LakeRawCdcEpochWritePlan) -> Result<()> {
    compare_plan_field(
        "dataset_id",
        &write_plan.dataset_id,
        &write_plan.epoch_metadata.epoch_row.dataset_id,
    )?;
    compare_plan_field(
        "epoch_id",
        &write_plan.epoch_id,
        &write_plan.epoch_metadata.epoch_row.epoch_id,
    )?;
    compare_plan_field(
        "transaction_count",
        &write_plan.transaction_count.to_string(),
        &write_plan
            .epoch_metadata
            .epoch_row
            .transaction_count
            .to_string(),
    )?;
    compare_plan_field(
        "change_count",
        &write_plan.change_count.to_string(),
        &write_plan.epoch_metadata.epoch_row.change_count.to_string(),
    )?;
    compare_plan_field(
        "checksum_rollup",
        &write_plan.checksum_rollup.to_string(),
        &write_plan
            .epoch_metadata
            .epoch_row
            .checksum_rollup
            .to_string(),
    )?;
    compare_plan_field(
        "data_file_count",
        &write_plan.data_file_count.to_string(),
        &write_plan.data_files.len().to_string(),
    )
}

fn compare_plan_field(field: &'static str, expected: &str, actual: &str) -> Result<()> {
    if expected == actual {
        return Ok(());
    }
    Err(IcebergIntegrationError::WritePlanBoundaryMismatch {
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}
