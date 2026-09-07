use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::{
    plan_iceberg_epoch_metadata_table_spec, plan_raw_cdc_iceberg_table_specs, IcebergCommitConfig,
    IcebergFaninL1TableSpecs, Result,
};

pub fn plan_iceberg_fanin_l1_table_specs(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<IcebergFaninL1TableSpecs> {
    Ok(IcebergFaninL1TableSpecs {
        dataset_id: write_plan.dataset_id.clone(),
        epoch_id: write_plan.epoch_id.clone(),
        completeness_contract: "append_only_raw_changelog_plus_queryable_trellara_epochs"
            .to_string(),
        raw_changelog_tables: plan_raw_cdc_iceberg_table_specs(write_plan, config)?,
        epoch_metadata_table: plan_iceberg_epoch_metadata_table_spec(write_plan, config)?,
    })
}
