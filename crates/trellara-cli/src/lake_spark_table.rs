use crate::{
    lake_table_prefix, selected_configured_tables, CliError, LakeSparkTemplateKind, Result,
    TableConfig, TrellaraConfig,
};

pub(crate) fn selected_lake_spark_table<'a>(
    config: &'a TrellaraConfig,
    table_filter: Option<&str>,
) -> Result<&'a TableConfig> {
    let selected = selected_configured_tables(
        &config.dataset.tables,
        table_filter,
        "lake spark-template --table",
    )?;
    selected.into_iter().next().ok_or_else(|| {
        CliError::InvalidConfig("dataset.tables must contain at least one table".to_string())
    })
}

pub(crate) fn lake_spark_target_table(
    dataset_id: &str,
    table_prefix: &str,
    template: LakeSparkTemplateKind,
) -> String {
    match template {
        LakeSparkTemplateKind::CurrentState => format!("{table_prefix}__current"),
        LakeSparkTemplateKind::Scd2 => format!("{table_prefix}__scd2"),
        LakeSparkTemplateKind::Maintenance => format!("{table_prefix}__current"),
        LakeSparkTemplateKind::Dashboard => lake_epoch_sources_table_name(dataset_id),
    }
}

pub(crate) fn lake_spark_raw_cdc_table(table_prefix: &str) -> String {
    format!("{table_prefix}__raw_cdc")
}

fn lake_epoch_sources_table_name(dataset_id: &str) -> String {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    format!("{prefix}___trellara_epoch_sources")
}
