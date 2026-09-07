use std::path::PathBuf;

use crate::{
    live_evidence_artifact_name, live_evidence_collection_requirements,
    live_evidence_expected_markers, CliError, FleetEvidencePlanArgs, FleetEvidencePlanFlow,
    FleetEvidencePlanGate, FleetEvidencePlanSummary, PilotScorecardGate, PilotScorecardStatus,
    PilotScorecardSummary, Result, TrellaraConfig,
};

impl FleetEvidencePlanSummary {
    pub(crate) fn from_args(args: &FleetEvidencePlanArgs) -> Result<Self> {
        if args.config.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet evidence-plan requires at least one --config".to_string(),
            ));
        }

        let mut configs = Vec::new();
        for path in &args.config {
            let config = TrellaraConfig::from_path(path)?;
            config.validate()?;
            configs.push((config, path.clone()));
        }

        Ok(Self::from_configs(&configs))
    }

    pub(crate) fn from_configs(configs: &[(TrellaraConfig, PathBuf)]) -> Self {
        let mut flows = Vec::new();
        let mut command_sequence = Vec::new();
        let mut gate_count = 0;
        let mut needs_live_evidence_gate_count = 0;
        let mut blocked_gate_count = 0;
        let mut required_live_evidence_artifact_count = 0;

        for (config, path) in configs {
            let scorecard = PilotScorecardSummary::from_config(config, path);
            gate_count += scorecard.gate_count;
            needs_live_evidence_gate_count += scorecard.needs_live_evidence_gate_count;
            blocked_gate_count += scorecard.blocked_gate_count;

            let live_evidence_gates = scorecard
                .gates
                .iter()
                .filter(|gate| gate.status == PilotScorecardStatus::NeedsLiveEvidence)
                .map(FleetEvidencePlanGate::from_scorecard_gate)
                .collect::<Vec<_>>();
            let blocked_gates = scorecard
                .gates
                .iter()
                .filter(|gate| gate.status == PilotScorecardStatus::Blocked)
                .map(FleetEvidencePlanGate::from_scorecard_gate)
                .collect::<Vec<_>>();
            required_live_evidence_artifact_count += live_evidence_gates.len();
            command_sequence.extend(
                live_evidence_gates
                    .iter()
                    .map(|gate| gate.proof_command.clone()),
            );
            command_sequence.push(format!(
                "trellara pilot evidence-template --config {} --output live-evidence --format text",
                path.display()
            ));
            command_sequence.push(format!(
                "trellara pilot evidence-check --config {} --evidence-dir live-evidence --format text",
                path.display()
            ));
            command_sequence.push(format!(
                "trellara pilot-package --config {}",
                path.display()
            ));
            command_sequence.push(
                "trellara evidence-registry --package target/trellara-pilot-package --format text"
                    .to_string(),
            );

            flows.push(FleetEvidencePlanFlow {
                flow_id: format!("{}:{}", config.source.id, config.dataset.id),
                config: path.display().to_string(),
                verdict: scorecard.verdict,
                needs_live_evidence_gate_count: live_evidence_gates.len(),
                blocked_gate_count: blocked_gates.len(),
                required_live_evidence_artifact_count: live_evidence_gates.len(),
                live_evidence_gates,
                blocked_gates,
            });
        }

        command_sequence.sort();
        command_sequence.dedup();
        let verdict = if blocked_gate_count > 0 {
            "blocked_until_config_fixed"
        } else if needs_live_evidence_gate_count > 0 {
            "ready_to_collect_live_evidence"
        } else {
            "live_evidence_complete"
        }
        .to_string();
        let config_flags = configs
            .iter()
            .map(|(_, path)| format!("--config {}", path.display()))
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            verdict,
            flow_count: flows.len(),
            gate_count,
            needs_live_evidence_gate_count,
            blocked_gate_count,
            required_live_evidence_artifact_count,
            command_count: command_sequence.len(),
            flows,
            command_sequence,
            review_rule:
                "collect every needs_live_evidence command before declaring fleet readiness; fix blocked gates before running live CDC"
                    .to_string(),
            next_commands: vec![
                format!("trellara fleet evidence-plan {config_flags} --format text"),
                format!("trellara fleet scorecard {config_flags} --format text"),
                format!("trellara fleet control-plane {config_flags} --format text"),
            ],
        }
    }
}

impl FleetEvidencePlanGate {
    pub(crate) fn from_scorecard_gate(gate: &PilotScorecardGate) -> Self {
        Self {
            code: gate.code.clone(),
            title: gate.name.clone(),
            artifact: live_evidence_artifact_name(&gate.code).to_string(),
            proof_command: gate.proof_command.clone(),
            success_evidence: gate.acceptance.clone(),
            success_markers: live_evidence_expected_markers(&gate.code),
            collection_requirements: live_evidence_collection_requirements(&gate.code),
        }
    }
}
