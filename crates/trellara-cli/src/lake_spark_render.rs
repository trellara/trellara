use crate::{lake_table_prefix, LakeSparkTemplateKind};
use sha2::{Digest, Sha256};

#[derive(Copy, Clone)]
pub(crate) struct LakeSparkTemplateRenderParams<'a> {
    pub(crate) catalog: &'a str,
    pub(crate) namespace: &'a str,
    pub(crate) raw_cdc_table: &'a str,
    pub(crate) epochs_table: &'a str,
    pub(crate) epoch_partitions_table: &'a str,
    pub(crate) verification_table: &'a str,
    pub(crate) quarantine_table: &'a str,
    pub(crate) target_table: &'a str,
    pub(crate) epoch_id: &'a str,
    pub(crate) primary_key_column: &'a str,
    pub(crate) accept_complete_with_gaps: bool,
    pub(crate) unsafe_allow_non_consumable_epoch: bool,
    pub(crate) unsafe_override_reason: &'a str,
}

pub(crate) fn lake_epochs_table_name(dataset_id: &str) -> String {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    format!("{prefix}___trellara_epochs")
}

pub(crate) fn lake_epoch_partitions_table_name(dataset_id: &str) -> String {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    format!("{prefix}___trellara_epoch_partitions")
}

pub(crate) fn lake_verification_table_name(dataset_id: &str) -> String {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    format!("{prefix}___trellara_verification")
}

pub(crate) fn lake_quarantine_table_name(dataset_id: &str) -> String {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    format!("{prefix}___trellara_quarantine")
}

pub(crate) fn render_lake_spark_template_sql(
    template: LakeSparkTemplateKind,
    params: LakeSparkTemplateRenderParams<'_>,
) -> String {
    let template_sql = match template {
        LakeSparkTemplateKind::CurrentState => {
            include_str!("../../../examples/spark/current_state.sql")
        }
        LakeSparkTemplateKind::Scd2 => include_str!("../../../examples/spark/scd2.sql"),
        LakeSparkTemplateKind::Maintenance => {
            include_str!("../../../examples/spark/maintenance.sql")
        }
        LakeSparkTemplateKind::Dashboard => {
            include_str!("../../../examples/spark/completeness_dashboard.sql")
        }
    };

    template_sql
        .replace("${catalog}", params.catalog)
        .replace("${namespace}", params.namespace)
        .replace("${raw_cdc_table}", params.raw_cdc_table)
        .replace("${epochs_table}", params.epochs_table)
        .replace("${epoch_partitions_table}", params.epoch_partitions_table)
        .replace("${verification_table}", params.verification_table)
        .replace("${quarantine_table}", params.quarantine_table)
        .replace("${target_table}", params.target_table)
        .replace("${epoch_id}", params.epoch_id)
        .replace("${primary_key_column}", params.primary_key_column)
        .replace(
            "${accept_complete_with_gaps}",
            if params.accept_complete_with_gaps {
                "true"
            } else {
                "false"
            },
        )
        .replace(
            "${unsafe_allow_non_consumable_epoch}",
            if params.unsafe_allow_non_consumable_epoch {
                "true"
            } else {
                "false"
            },
        )
        .replace("${unsafe_override_reason}", params.unsafe_override_reason)
}

pub(crate) fn lake_spark_template_sha256(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

#[cfg(test)]
#[path = "lake_spark_render_tests.rs"]
mod lake_spark_render_tests;
