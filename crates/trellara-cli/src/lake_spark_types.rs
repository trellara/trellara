use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeSparkTemplateSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) template: LakeSparkTemplateKind,
    pub(crate) relation: String,
    pub(crate) catalog: String,
    pub(crate) namespace: String,
    pub(crate) raw_cdc_table: String,
    pub(crate) epochs_table: String,
    pub(crate) epoch_partitions_table: String,
    pub(crate) verification_table: String,
    pub(crate) quarantine_table: String,
    pub(crate) target_table: String,
    pub(crate) epoch_id: String,
    pub(crate) primary_key_column: String,
    pub(crate) accept_complete_with_gaps: bool,
    pub(crate) unsafe_allow_non_consumable_epoch: bool,
    pub(crate) unsafe_override_reason: Option<String>,
    pub(crate) visibility_rule: String,
    pub(crate) idempotency_rule: String,
    pub(crate) template_sha256: String,
    pub(crate) sql: String,
    pub(crate) pyspark_runner: String,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakeSparkTemplateKind {
    CurrentState,
    Scd2,
    Maintenance,
    Dashboard,
}
