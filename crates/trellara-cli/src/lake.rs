use crate::lake_plan_materializations::lake_spark_template_outputs;
use crate::{
    lake_epoch_metadata_ddl_tables, lake_raw_cdc_table, DatasetMode, LakeDdlCheckpointContract,
    LakeDdlSummary, TrellaraConfig,
};

impl LakeDdlSummary {
    pub(crate) fn from_config(config: &TrellaraConfig) -> Self {
        let visibility_boundary = lake_visibility_boundary_message(config).to_string();
        let uses_ownership_key = config.dataset.partition.is_some();
        let mut tables: Vec<_> = config
            .dataset
            .tables
            .iter()
            .map(|table| {
                lake_raw_cdc_table(
                    &config.dataset.id,
                    table,
                    &visibility_boundary,
                    uses_ownership_key,
                )
            })
            .collect();
        tables.extend(lake_epoch_metadata_ddl_tables(
            &config.dataset.id,
            &visibility_boundary,
        ));

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            visibility_boundary: visibility_boundary.clone(),
            table_count: tables.len(),
            tables,
            spark_template_outputs: lake_spark_template_outputs(&config.dataset.id),
            checkpoint_contract: LakeDdlCheckpointContract {
                checkpoint_column: "__trellara_commit_lsn".to_string(),
                checkpoint_source: "source transaction commit_lsn after envelope checksum verification"
                    .to_string(),
                visibility_rule: format!(
                    "publish lake epochs only after {visibility_boundary}"
                ),
                idempotency_rule:
                    "deduplicate raw CDC rows by __trellara_idempotency_key; Spark-derived current/SCD2 jobs consume only completed epoch ids by default"
                        .to_string(),
            },
            recommended_next_steps: vec![
                "review generated table names with the data platform owner".to_string(),
                "pin source schema fingerprints before enabling lake writes".to_string(),
                "generate Spark templates for current-state and SCD2 from completed epochs".to_string(),
                "run trellara lake inspect --config <flow.yml> --file <envelope.pb> against a captured transaction"
                    .to_string(),
                "run trellara lake fanin verify after the first raw CDC and epoch metadata rehearsal"
                    .to_string(),
            ],
        }
    }
}

pub(crate) fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn lake_visibility_boundary_message(config: &TrellaraConfig) -> &'static str {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict chunk manifest and commit marker barrier is the visibility boundary"
        }
        DatasetMode::StrictTransactionOrder => {
            "source transaction envelope is the visibility boundary"
        }
        DatasetMode::PartitionedScaleMode => {
            "manifest and commit marker barrier plus global low watermark is the visibility boundary"
        }
    }
}

pub(crate) fn lake_table_prefix(dataset_id: &str, schema: &str, table: &str) -> String {
    [dataset_id, schema, table]
        .into_iter()
        .map(lake_identifier_segment)
        .collect::<Vec<_>>()
        .join("__")
}

pub(crate) fn lake_identifier_segment(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('_') {
            output.push('_');
        }
    }
    output.trim_matches('_').to_string()
}
