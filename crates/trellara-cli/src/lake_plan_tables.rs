use crate::{lake_table_prefix, LakeTablePlan, TableConfig, TrellaraConfig};

pub(crate) fn lake_materialization_config(
    config: &TrellaraConfig,
) -> trellara_lake::LakePlanConfig {
    trellara_lake::LakePlanConfig::new(
        config
            .dataset
            .tables
            .iter()
            .map(|table| {
                lake_table_config(
                    table,
                    config
                        .dataset
                        .partition
                        .as_ref()
                        .map(|partition| partition.key_column.clone()),
                )
            })
            .collect(),
    )
}

pub(crate) fn lake_table_plans(config: &TrellaraConfig) -> Vec<LakeTablePlan> {
    config
        .dataset
        .tables
        .iter()
        .map(|table| lake_table_plan(&config.dataset.id, table))
        .collect()
}

fn lake_table_config(
    table: &TableConfig,
    partition_key_column: Option<String>,
) -> trellara_lake::LakeTableConfig {
    let verify = table.verify.clone().unwrap_or_default();
    let contract = table.contract.clone().unwrap_or_default();
    let mut lake_table =
        trellara_lake::LakeTableConfig::new(&table.schema, &table.name, verify.primary_key)
            .with_source_schema_fingerprint(contract.source_schema_fingerprint)
            .with_partition_key_column(partition_key_column);
    for column in verify.excluded_columns {
        lake_table = lake_table.excluding(column);
    }
    lake_table
}

fn lake_table_plan(dataset_id: &str, table: &TableConfig) -> LakeTablePlan {
    let verify = table.verify.clone().unwrap_or_default();
    let contract = table.contract.clone().unwrap_or_default();
    let relation = table.relation_id().display_name();
    let table_prefix = lake_table_prefix(dataset_id, &table.schema, &table.name);
    let mut notes = vec![
        "raw CDC preserves every committed row event with source transaction metadata".to_string(),
        "current-state and SCD2 are Spark-derived from completed lake epochs in the first production fan-in path".to_string(),
    ];
    if !contract.target_owned_columns.is_empty() {
        notes.push(
            "target-owned operational columns stay out of lake materializations unless explicitly modeled"
                .to_string(),
        );
    }
    if verify.row_filter.is_some() {
        notes.push(
            "row filter scopes validation and reseed; lake writers should carry the same filter contract"
                .to_string(),
        );
    }

    LakeTablePlan {
        relation,
        primary_key: verify.primary_key,
        source_schema_fingerprint: contract.source_schema_fingerprint,
        excluded_columns: verify.excluded_columns,
        row_filter: verify.row_filter,
        target_owned_columns: contract.target_owned_columns,
        raw_cdc_table: format!("{table_prefix}__raw_cdc"),
        current_state_template_output: format!("{table_prefix}__current"),
        scd2_history_template_output: format!("{table_prefix}__scd2"),
        notes,
    }
}
