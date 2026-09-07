use serde::Serialize;

pub(crate) use crate::ddl_propagation_types::*;
use crate::DdlPlanApplyMode;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPlanSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) apply_mode: DdlPlanApplyMode,
    pub(crate) unknown_table_policy: String,
    pub(crate) proposed_change_count: usize,
    pub(crate) auto_apply_count: usize,
    pub(crate) staged_rollout_count: usize,
    pub(crate) manual_review_count: usize,
    pub(crate) blocked_count: usize,
    pub(crate) blockers: Vec<DdlPlanBlocker>,
    pub(crate) verdict: DdlPlanVerdict,
    pub(crate) transaction_boundary_rule: String,
    pub(crate) propagation: DdlPropagationPlan,
    pub(crate) changes: Vec<DdlPlanChange>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPlanBlocker {
    pub(crate) change: String,
    pub(crate) kind: DdlPlanChangeKind,
    pub(crate) object: String,
    pub(crate) relation: Option<String>,
    pub(crate) compatibility: DdlPlanCompatibility,
    pub(crate) reason: String,
    pub(crate) release_impact: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlApplyPlanSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) barrier_id: String,
    pub(crate) executable: bool,
    pub(crate) transaction_boundary_rule: String,
    pub(crate) plan_sha256: String,
    pub(crate) blocker_count: usize,
    pub(crate) blockers: Vec<String>,
    pub(crate) statement_count: usize,
    pub(crate) statements: Vec<DdlApplyPlanStatement>,
    pub(crate) target_postgres_transaction_script: Option<String>,
    pub(crate) target_postgres_ack_commands: Vec<String>,
    pub(crate) steps: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlApplyPlanStatement {
    pub(crate) change: String,
    pub(crate) sink: String,
    pub(crate) sql: String,
    pub(crate) statement_sha256: String,
    pub(crate) transaction_scope: String,
    pub(crate) release_gate_code: String,
    pub(crate) release_gate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPlanChange {
    pub(crate) input: String,
    pub(crate) kind: DdlPlanChangeKind,
    pub(crate) object: String,
    pub(crate) relation: Option<String>,
    pub(crate) configured_relation: bool,
    pub(crate) compatibility: DdlPlanCompatibility,
    pub(crate) decision: DdlPlanDecision,
    pub(crate) reason: String,
    pub(crate) boundary_rule: String,
    pub(crate) target_postgres_sql: Option<String>,
    pub(crate) propagation_actions: Vec<DdlPropagationAction>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPlanChangeKind {
    AddNullableColumn,
    AddTable,
    WidenType,
    IncreaseVarchar,
    DropColumn,
    RenameColumn,
    RenameTable,
    NarrowType,
    ChangePrimaryKey,
    ChangePartitionKey,
    AddNotNullColumn,
    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPlanCompatibility {
    Compatible,
    RequiresMapping,
    DestructiveOrAmbiguous,
    BlockedByPolicy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPlanDecision {
    AutoApply,
    StageThenApply,
    ManualReview,
    Block,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPlanVerdict {
    ReadyToApply,
    RequiresManualReview,
    Blocked,
}
