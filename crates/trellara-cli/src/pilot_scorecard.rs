use std::path::Path;

use crate::{
    pilot_scorecard_gates, pilot_scorecard_next_commands, DatasetMode, PilotScorecardGate,
    PilotScorecardStatus, PilotScorecardSummary, StreamConfig, TrellaraConfig,
};

impl PilotScorecardSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let local_stream = matches!(config.stream, StreamConfig::Local { .. });
        let has_target = config.target.is_some();
        let partitioned = config.dataset.mode == DatasetMode::PartitionedScaleMode;

        let gates = pilot_scorecard_gates(config, &config_path, local_stream, has_target);

        let gate_count = gates.len();
        let configuration_ready_gate_count = gates
            .iter()
            .filter(|gate| gate.status == PilotScorecardStatus::ConfigurationReady)
            .count();
        let needs_live_evidence_gate_count = gates
            .iter()
            .filter(|gate| gate.status == PilotScorecardStatus::NeedsLiveEvidence)
            .count();
        let blocked_gate_count = gates
            .iter()
            .filter(|gate| gate.status == PilotScorecardStatus::Blocked)
            .count();
        let non_blocked_count = gates
            .iter()
            .filter(|gate| gate.status != PilotScorecardStatus::Blocked)
            .count();
        let score = ((non_blocked_count * 100) / gate_count) as u8;
        let verdict = if blocked_gate_count == 0 {
            "ready_for_live_pilot"
        } else {
            "blocked_before_verified_pilot"
        }
        .to_string();

        let next_commands =
            pilot_scorecard_next_commands(&config_path, local_stream, has_target, partitioned);

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path,
            verdict,
            score,
            gate_count,
            configuration_ready_gate_count,
            needs_live_evidence_gate_count,
            blocked_gate_count,
            gates,
            next_commands,
        }
    }
}

impl PilotScorecardGate {
    pub(crate) fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        status: PilotScorecardStatus,
        evidence: impl Into<String>,
        proof_command: impl Into<String>,
        acceptance: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            status,
            evidence: evidence.into(),
            proof_command: proof_command.into(),
            acceptance: acceptance.into(),
        }
    }
}
