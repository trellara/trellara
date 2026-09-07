use crate::{
    lake_materialization_boundary, lake_table_prefix, LakeMaterializationSummary, TrellaraConfig,
};

pub(crate) fn lake_materializations(config: &TrellaraConfig) -> Vec<LakeMaterializationSummary> {
    let boundary = lake_materialization_boundary(config);

    vec![
        LakeMaterializationSummary {
            kind: "raw_cdc_append_only".to_string(),
            visibility_boundary: boundary.to_string(),
            purpose: "append-only Iceberg history for every committed row event with Trellara transaction and epoch metadata".to_string(),
            producer: "trellara_lake_writer".to_string(),
        },
        LakeMaterializationSummary {
            kind: "epoch_metadata".to_string(),
            visibility_boundary: boundary.to_string(),
            purpose: "queryable completeness tables for epochs, sources, partition rollups, table rollups, quarantine, and verification".to_string(),
            producer: "trellara_lake_writer".to_string(),
        },
        LakeMaterializationSummary {
            kind: "current_state_template".to_string(),
            visibility_boundary: boundary.to_string(),
            purpose: "Spark-derived latest-row table generated only from complete or explicitly gap-accepted epochs".to_string(),
            producer: "spark_template".to_string(),
        },
        LakeMaterializationSummary {
            kind: "scd2_history_template".to_string(),
            visibility_boundary: boundary.to_string(),
            purpose: "Spark-derived valid-time history generated only from complete or explicitly gap-accepted epochs".to_string(),
            producer: "spark_template".to_string(),
        },
    ]
}

pub(crate) fn lake_epoch_metadata_tables() -> Vec<String> {
    vec![
        "_trellara_epochs".to_string(),
        "_trellara_epoch_sources".to_string(),
        "_trellara_epoch_tables".to_string(),
        "_trellara_epoch_partitions".to_string(),
        "_trellara_quarantine".to_string(),
        "_trellara_verification".to_string(),
    ]
}

pub(crate) fn lake_spark_template_outputs(dataset_id: &str) -> Vec<String> {
    let prefix = lake_table_prefix(dataset_id, "spark", "derived");
    vec![
        format!("{prefix}__current_state"),
        format!("{prefix}__scd2_history"),
    ]
}
