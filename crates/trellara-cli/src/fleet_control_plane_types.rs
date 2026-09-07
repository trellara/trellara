use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetControlPlaneSummary {
    pub(crate) verdict: String,
    pub(crate) build_recommendation: String,
    pub(crate) fleet_shape: FleetControlPlaneShape,
    pub(crate) capability_pull_count: usize,
    pub(crate) capabilities_to_build: Vec<FleetControlPlaneCapability>,
    pub(crate) defer_until_pulled: Vec<String>,
    pub(crate) evidence_gaps: Vec<String>,
    pub(crate) identity_collisions: Vec<String>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetControlPlaneShape {
    pub(crate) flow_count: usize,
    pub(crate) source_count: usize,
    pub(crate) dataset_count: usize,
    pub(crate) table_count: usize,
    pub(crate) local_stream_count: usize,
    pub(crate) kafka_stream_count: usize,
    pub(crate) partitioned_flow_count: usize,
    pub(crate) target_configured_count: usize,
    pub(crate) topology_verdict: String,
    pub(crate) fleet_scorecard_verdict: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetControlPlaneCapability {
    pub(crate) code: String,
    pub(crate) status: FleetControlPlaneCapabilityStatus,
    pub(crate) rationale: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) build_when: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FleetControlPlaneCapabilityStatus {
    PulledNow,
    ValidateWithPartners,
    Defer,
}
