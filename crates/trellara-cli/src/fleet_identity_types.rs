use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetIdentityAuditSummary {
    pub(crate) verdict: String,
    pub(crate) flow_count: usize,
    pub(crate) unique_flow_count: usize,
    pub(crate) duplicate_flow_count: usize,
    pub(crate) source_count: usize,
    pub(crate) dataset_count: usize,
    pub(crate) flows: Vec<FleetIdentityFlow>,
    pub(crate) duplicate_groups: Vec<FleetIdentityDuplicateGroup>,
    pub(crate) remediation: Vec<String>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetIdentityFlow {
    pub(crate) flow_id: String,
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) stream_kind: String,
    pub(crate) status: FleetIdentityStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetIdentityDuplicateGroup {
    pub(crate) flow_id: String,
    pub(crate) configs: Vec<String>,
    pub(crate) remediation: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FleetIdentityStatus {
    Unique,
    Duplicate,
}
