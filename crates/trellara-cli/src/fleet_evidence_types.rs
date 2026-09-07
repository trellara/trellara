use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetEvidencePlanSummary {
    pub(crate) verdict: String,
    pub(crate) flow_count: usize,
    pub(crate) gate_count: usize,
    pub(crate) needs_live_evidence_gate_count: usize,
    pub(crate) blocked_gate_count: usize,
    pub(crate) required_live_evidence_artifact_count: usize,
    pub(crate) command_count: usize,
    pub(crate) flows: Vec<FleetEvidencePlanFlow>,
    pub(crate) command_sequence: Vec<String>,
    pub(crate) review_rule: String,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetEvidencePlanFlow {
    pub(crate) flow_id: String,
    pub(crate) config: String,
    pub(crate) verdict: String,
    pub(crate) needs_live_evidence_gate_count: usize,
    pub(crate) blocked_gate_count: usize,
    pub(crate) required_live_evidence_artifact_count: usize,
    pub(crate) live_evidence_gates: Vec<FleetEvidencePlanGate>,
    pub(crate) blocked_gates: Vec<FleetEvidencePlanGate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetEvidencePlanGate {
    pub(crate) code: String,
    pub(crate) title: String,
    pub(crate) artifact: String,
    pub(crate) proof_command: String,
    pub(crate) success_evidence: String,
    pub(crate) success_markers: Vec<String>,
    pub(crate) collection_requirements: Vec<String>,
}
