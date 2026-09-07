use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetConvergenceGate {
    pub(crate) code: String,
    pub(crate) status: FleetConvergenceGateStatus,
    pub(crate) evidence: String,
    pub(crate) proof_command: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FleetConvergenceGateStatus {
    NeedsLiveEvidence,
    BlockedByConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetRecoveryDrill {
    pub(crate) code: String,
    pub(crate) trigger: String,
    pub(crate) operator_goal: String,
    pub(crate) commands: Vec<String>,
    pub(crate) success_evidence: String,
}
