use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPropagationPlan {
    pub(crate) barrier_id: String,
    pub(crate) barrier_scope: String,
    pub(crate) cdc_transaction_boundary: String,
    pub(crate) row_visibility_mode: String,
    pub(crate) dml_after_barrier_held: bool,
    pub(crate) requires_global_partition_pause: bool,
    pub(crate) sink_count: usize,
    pub(crate) required_ack_count: usize,
    pub(crate) ack_quorum: String,
    pub(crate) policy_modes: Vec<DdlPropagationPolicySummary>,
    pub(crate) release_blockers: Vec<String>,
    pub(crate) sinks: Vec<DdlPropagationSink>,
    pub(crate) phases: Vec<DdlPropagationPhase>,
    pub(crate) release_gates: Vec<DdlReleaseGate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPropagationPolicySummary {
    pub(crate) mode: DdlPropagationPolicyMode,
    pub(crate) active: bool,
    pub(crate) change_count: usize,
    pub(crate) release_rule: String,
    pub(crate) approval_evidence: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPropagationPolicyMode {
    AutoApply,
    StagedRollout,
    ManualApprovalRequired,
    BlockUnsupported,
    ShadowPlanOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPropagationSink {
    pub(crate) name: String,
    pub(crate) kind: DdlPropagationSinkKind,
    pub(crate) required_ack: String,
    pub(crate) ack_evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPropagationPhase {
    pub(crate) order: u32,
    pub(crate) name: String,
    pub(crate) release_condition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlReleaseGate {
    pub(crate) name: String,
    pub(crate) required_evidence: String,
    pub(crate) opens_when: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlPropagationAction {
    pub(crate) sink: String,
    pub(crate) action: String,
    pub(crate) ack_condition: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DdlPropagationSinkKind {
    TargetPostgres,
    RawCdcLake,
    SparkDerivedView,
    PartitionVisibility,
}
