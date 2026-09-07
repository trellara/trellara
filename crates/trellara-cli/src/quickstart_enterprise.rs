use std::path::Path;

use serde::Serialize;

use crate::{
    enterprise_blockers, enterprise_buyer_summary, enterprise_differentiators,
    enterprise_live_evidence_required, enterprise_mode_contract, enterprise_next_commands,
    enterprise_proof_commands, enterprise_readiness_gates, enterprise_recommended_mode,
    enterprise_transaction_boundary_contract, EnterpriseReadinessGate, PilotScorecardSummary,
    StreamConfig, TrellaraConfig,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EnterpriseEvaluationSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) verdict: String,
    pub(crate) recommended_mode: String,
    pub(crate) buyer_summary: String,
    pub(crate) mode_contract: String,
    pub(crate) transaction_boundary_contract: String,
    pub(crate) public_modes: Vec<String>,
    pub(crate) differentiators: Vec<String>,
    pub(crate) readiness_gates: Vec<EnterpriseReadinessGate>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) live_evidence_required: Vec<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) scorecard_gate_count: usize,
    pub(crate) configuration_ready_gate_count: usize,
    pub(crate) needs_live_evidence_gate_count: usize,
    pub(crate) blocked_gate_count: usize,
    pub(crate) next_commands: Vec<String>,
}

impl EnterpriseEvaluationSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let scorecard = PilotScorecardSummary::from_config(config, path);
        let local_stream = matches!(config.stream, StreamConfig::Local { .. });
        let has_target = config.target.is_some();

        let recommended_mode = enterprise_recommended_mode(config);
        let mode_contract = enterprise_mode_contract(config);
        let transaction_boundary_contract = enterprise_transaction_boundary_contract(config);
        let buyer_summary = enterprise_buyer_summary(config, &recommended_mode, has_target);

        let proof_commands = enterprise_proof_commands(config, &config_path, local_stream);
        let differentiators = enterprise_differentiators(config);
        let live_evidence_required = enterprise_live_evidence_required(config, local_stream);
        let blockers = enterprise_blockers(&scorecard, has_target);
        let readiness_gates = enterprise_readiness_gates(config, &config_path, local_stream);

        let verdict = if blockers.is_empty() {
            "ready_for_enterprise_pilot_evidence"
        } else {
            "blocked_before_enterprise_pilot"
        }
        .to_string();

        let next_commands =
            enterprise_next_commands(config, &config_path, local_stream, has_target);

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path,
            verdict,
            recommended_mode,
            buyer_summary,
            mode_contract,
            transaction_boundary_contract,
            public_modes: vec![
                "strict_transaction_order".to_string(),
                "partitioned_scale_mode".to_string(),
            ],
            differentiators,
            readiness_gates,
            proof_commands,
            live_evidence_required,
            blockers,
            scorecard_gate_count: scorecard.gate_count,
            configuration_ready_gate_count: scorecard.configuration_ready_gate_count,
            needs_live_evidence_gate_count: scorecard.needs_live_evidence_gate_count,
            blocked_gate_count: scorecard.blocked_gate_count,
            next_commands,
        }
    }
}
