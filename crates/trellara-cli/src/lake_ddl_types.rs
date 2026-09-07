use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeDdlSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) contract: String,
    pub(crate) visibility_boundary: String,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<LakeDdlTable>,
    pub(crate) spark_template_outputs: Vec<String>,
    pub(crate) checkpoint_contract: LakeDdlCheckpointContract,
    pub(crate) recommended_next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeDdlTable {
    pub(crate) relation: String,
    pub(crate) materialization: String,
    pub(crate) table_name: String,
    pub(crate) primary_key: Option<String>,
    pub(crate) checkpoint_column: String,
    pub(crate) visibility_boundary: String,
    pub(crate) partitioning: Vec<String>,
    pub(crate) ddl: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeDdlCheckpointContract {
    pub(crate) checkpoint_column: String,
    pub(crate) checkpoint_source: String,
    pub(crate) visibility_rule: String,
    pub(crate) idempotency_rule: String,
}
