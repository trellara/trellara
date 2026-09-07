use crate::lake_plan_materializations::{
    lake_epoch_metadata_tables, lake_materializations, lake_spark_template_outputs,
};
use crate::lake_plan_tables::lake_table_plans;
use crate::{
    lake_plan_checks, lake_recommended_next_steps, DatasetMode, LakePlanCheckStatus,
    LakePlanSummary, TrellaraConfig,
};

impl LakePlanSummary {
    pub(crate) fn from_config(config: &TrellaraConfig) -> Self {
        let tables = lake_table_plans(config);
        let checks = lake_plan_checks(config, &tables);
        let blocking_check_count = checks
            .iter()
            .filter(|check| check.status == LakePlanCheckStatus::Blocked)
            .count();
        let warning_check_count = checks
            .iter()
            .filter(|check| check.status == LakePlanCheckStatus::Warning)
            .count();

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            fanin_mode: lake_fanin_mode(config).to_string(),
            straggler_policy: "wait_all_required by default; publish_with_gaps and quarantine_on_gap are explicit epoch policies".to_string(),
            native_writer_scope:
                "Trellara writes append-only raw CDC and epoch metadata; current-state and SCD2 are Spark-derived templates until native Iceberg deletes mature"
                    .to_string(),
            materialization_count: 4,
            materializations: lake_materializations(config),
            epoch_metadata_tables: lake_epoch_metadata_tables(),
            spark_template_outputs: lake_spark_template_outputs(&config.dataset.id),
            table_count: tables.len(),
            tables,
            check_count: checks.len(),
            blocking_check_count,
            warning_check_count,
            recommended_next_steps: lake_recommended_next_steps(
                blocking_check_count,
                warning_check_count,
            ),
            checks,
        }
    }
}

pub(crate) fn lake_fanin_mode(config: &TrellaraConfig) -> &'static str {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict_chunked_epoch_fanin"
        }
        DatasetMode::StrictTransactionOrder => "strict_envelope_epoch_fanin",
        DatasetMode::PartitionedScaleMode => "partitioned_scale_epoch_fanin",
    }
}
