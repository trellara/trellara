use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakePlanSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) contract: String,
    pub(crate) fanin_mode: String,
    pub(crate) straggler_policy: String,
    pub(crate) native_writer_scope: String,
    pub(crate) materialization_count: usize,
    pub(crate) materializations: Vec<LakeMaterializationSummary>,
    pub(crate) epoch_metadata_tables: Vec<String>,
    pub(crate) spark_template_outputs: Vec<String>,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<LakeTablePlan>,
    pub(crate) check_count: usize,
    pub(crate) blocking_check_count: usize,
    pub(crate) warning_check_count: usize,
    pub(crate) checks: Vec<LakePlanCheck>,
    pub(crate) recommended_next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeMaterializationSummary {
    pub(crate) kind: String,
    pub(crate) visibility_boundary: String,
    pub(crate) purpose: String,
    pub(crate) producer: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeTablePlan {
    pub(crate) relation: String,
    pub(crate) primary_key: String,
    pub(crate) source_schema_fingerprint: Option<u64>,
    pub(crate) excluded_columns: Vec<String>,
    pub(crate) row_filter: Option<String>,
    pub(crate) target_owned_columns: Vec<String>,
    pub(crate) raw_cdc_table: String,
    pub(crate) current_state_template_output: String,
    pub(crate) scd2_history_template_output: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakePlanCheck {
    pub(crate) name: String,
    pub(crate) status: LakePlanCheckStatus,
    pub(crate) message: String,
    pub(crate) recommendation: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakePlanCheckStatus {
    Ready,
    Warning,
    Blocked,
}
