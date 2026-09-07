use std::path::Path;

use serde::Serialize;

use crate::{
    pilot_evidence_counts::{next_evidence_commands, PilotLiveEvidenceGateCounts},
    pilot_evidence_identity_helpers::ExpectedIdentity,
    PilotScorecardStatus, PilotScorecardSummary, TrellaraConfig,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotLiveEvidenceCheckSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) evidence_dir: String,
    pub(crate) verdict: String,
    pub(crate) gate_count: usize,
    pub(crate) accepted_gate_count: usize,
    pub(crate) missing_gate_count: usize,
    pub(crate) insufficient_gate_count: usize,
    pub(crate) not_required_gate_count: usize,
    pub(crate) blocked_gate_count: usize,
    pub(crate) gates: Vec<PilotLiveEvidenceGate>,
    pub(crate) review_rule: String,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotEvidenceTemplateSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) output: String,
    pub(crate) artifact_count: usize,
    pub(crate) files: Vec<PilotEvidenceTemplateFile>,
    pub(crate) artifacts: Vec<PilotEvidenceTemplateArtifact>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotEvidenceTemplateFile {
    pub(crate) path: String,
    pub(crate) purpose: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotEvidenceTemplateArtifact {
    pub(crate) gate_code: String,
    pub(crate) artifact: String,
    pub(crate) proof_command: String,
    pub(crate) success_markers: Vec<String>,
    pub(crate) collection_requirements: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotLiveEvidenceGate {
    pub(crate) code: String,
    pub(crate) title: String,
    pub(crate) scorecard_status: PilotScorecardStatus,
    pub(crate) evidence_status: PilotLiveEvidenceStatus,
    pub(crate) artifact: Option<String>,
    pub(crate) proof_command: String,
    pub(crate) success_evidence: String,
    pub(crate) observed_markers: Vec<String>,
    pub(crate) missing_markers: Vec<String>,
    pub(crate) note: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PilotLiveEvidenceStatus {
    Accepted,
    Missing,
    Insufficient,
    NotRequired,
    Blocked,
}

impl PilotLiveEvidenceCheckSummary {
    pub(crate) fn from_config(
        config: &TrellaraConfig,
        config_path: &Path,
        evidence_dir: &Path,
    ) -> Self {
        let scorecard = PilotScorecardSummary::from_config(config, config_path);
        let expected_identity = ExpectedIdentity {
            source_id: &config.source.id,
            dataset_id: &config.dataset.id,
        };
        let gates = scorecard
            .gates
            .iter()
            .map(|gate| {
                PilotLiveEvidenceGate::from_scorecard_gate_with_identity(
                    gate,
                    evidence_dir,
                    expected_identity,
                )
            })
            .collect::<Vec<_>>();

        let counts = PilotLiveEvidenceGateCounts::from_gates(&gates);
        let next_commands = next_evidence_commands(
            &gates,
            format!(
                "trellara pilot evidence-check --config {} --evidence-dir {} --format text",
                config_path.display(),
                evidence_dir.display()
            ),
        );

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path.display().to_string(),
            evidence_dir: evidence_dir.display().to_string(),
            verdict: counts.verdict().to_string(),
            gate_count: gates.len(),
            accepted_gate_count: counts.accepted,
            missing_gate_count: counts.missing,
            insufficient_gate_count: counts.insufficient,
            not_required_gate_count: counts.not_required,
            blocked_gate_count: counts.blocked,
            gates,
            review_rule:
                "every needs_live_evidence gate must have an accepted artifact before declaring live pilot evidence complete"
                    .to_string(),
            next_commands,
        }
    }
}
