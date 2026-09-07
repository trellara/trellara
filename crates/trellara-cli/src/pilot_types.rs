use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotGuideSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) objective: String,
    pub(crate) evaluation_time_budget_minutes: u32,
    pub(crate) kafka_required: bool,
    pub(crate) stream_kind: String,
    pub(crate) transaction_boundary: String,
    pub(crate) capture_spill_boundary: String,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<String>,
    pub(crate) phase_count: usize,
    pub(crate) phases: Vec<PilotGuidePhase>,
    pub(crate) evidence_commands: Vec<String>,
    pub(crate) large_transaction_evidence: Vec<String>,
    pub(crate) failure_drill: Vec<String>,
    pub(crate) acceptance_gates: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotScorecardSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) verdict: String,
    pub(crate) score: u8,
    pub(crate) gate_count: usize,
    pub(crate) configuration_ready_gate_count: usize,
    pub(crate) needs_live_evidence_gate_count: usize,
    pub(crate) blocked_gate_count: usize,
    pub(crate) gates: Vec<PilotScorecardGate>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotScorecardGate {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) status: PilotScorecardStatus,
    pub(crate) evidence: String,
    pub(crate) proof_command: String,
    pub(crate) acceptance: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PilotScorecardStatus {
    ConfigurationReady,
    NeedsLiveEvidence,
    EnvironmentSpecific,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotGuidePhase {
    pub(crate) order: usize,
    pub(crate) name: String,
    pub(crate) command: String,
    pub(crate) proof: String,
}
