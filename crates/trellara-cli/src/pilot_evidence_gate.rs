use std::fs;
use std::path::Path;

use crate::{
    live_evidence_artifact_name, live_evidence_expected_markers, live_evidence_markers,
    pilot_evidence_identity_helpers::{source_dataset_identity_matches, ExpectedIdentity},
    PilotLiveEvidenceGate, PilotLiveEvidenceStatus, PilotScorecardGate, PilotScorecardStatus,
};

impl PilotLiveEvidenceGate {
    pub(crate) fn from_scorecard_gate_with_identity(
        gate: &PilotScorecardGate,
        evidence_dir: &Path,
        expected_identity: ExpectedIdentity<'_>,
    ) -> Self {
        match gate.status {
            PilotScorecardStatus::ConfigurationReady
            | PilotScorecardStatus::EnvironmentSpecific => Self {
                code: gate.code.clone(),
                title: gate.name.clone(),
                scorecard_status: gate.status,
                evidence_status: PilotLiveEvidenceStatus::NotRequired,
                artifact: None,
                proof_command: gate.proof_command.clone(),
                success_evidence: gate.acceptance.clone(),
                observed_markers: Vec::new(),
                missing_markers: Vec::new(),
                note: "scorecard gate is configuration-ready; no live artifact is required"
                    .to_string(),
            },
            PilotScorecardStatus::Blocked => Self {
                code: gate.code.clone(),
                title: gate.name.clone(),
                scorecard_status: gate.status,
                evidence_status: PilotLiveEvidenceStatus::Blocked,
                artifact: None,
                proof_command: gate.proof_command.clone(),
                success_evidence: gate.acceptance.clone(),
                observed_markers: Vec::new(),
                missing_markers: Vec::new(),
                note: "scorecard gate is blocked before live evidence can be accepted".to_string(),
            },
            PilotScorecardStatus::NeedsLiveEvidence => {
                live_evidence_gate_from_artifact(gate, evidence_dir, expected_identity)
            }
        }
    }
}

fn live_evidence_gate_from_artifact(
    gate: &PilotScorecardGate,
    evidence_dir: &Path,
    expected_identity: ExpectedIdentity<'_>,
) -> PilotLiveEvidenceGate {
    let artifact_name = live_evidence_artifact_name(&gate.code);
    let artifact_path = evidence_dir.join(artifact_name);
    match fs::read_to_string(&artifact_path) {
        Ok(contents) => {
            let (mut observed_markers, mut missing_markers) =
                live_evidence_markers(&gate.code, &contents);
            enforce_expected_identity(
                &contents,
                expected_identity,
                &mut observed_markers,
                &mut missing_markers,
            );
            let evidence_status = if missing_markers.is_empty() {
                PilotLiveEvidenceStatus::Accepted
            } else {
                PilotLiveEvidenceStatus::Insufficient
            };
            let note = if evidence_status == PilotLiveEvidenceStatus::Accepted {
                "artifact contains the gate-specific success markers".to_string()
            } else {
                "artifact exists but does not prove the gate-specific acceptance rule".to_string()
            };

            PilotLiveEvidenceGate {
                code: gate.code.clone(),
                title: gate.name.clone(),
                scorecard_status: gate.status,
                evidence_status,
                artifact: Some(artifact_path.display().to_string()),
                proof_command: gate.proof_command.clone(),
                success_evidence: gate.acceptance.clone(),
                observed_markers,
                missing_markers,
                note,
            }
        }
        Err(_) => PilotLiveEvidenceGate {
            code: gate.code.clone(),
            title: gate.name.clone(),
            scorecard_status: gate.status,
            evidence_status: PilotLiveEvidenceStatus::Missing,
            artifact: Some(artifact_path.display().to_string()),
            proof_command: gate.proof_command.clone(),
            success_evidence: gate.acceptance.clone(),
            observed_markers: Vec::new(),
            missing_markers: live_evidence_expected_markers(&gate.code),
            note: "expected live evidence artifact is missing".to_string(),
        },
    }
}

fn enforce_expected_identity(
    contents: &str,
    expected: ExpectedIdentity<'_>,
    observed_markers: &mut Vec<String>,
    missing_markers: &mut Vec<String>,
) {
    if !observed_markers
        .iter()
        .any(|marker| marker == "source dataset identity")
    {
        return;
    }
    if source_dataset_identity_matches(contents, expected) {
        return;
    }
    observed_markers.retain(|marker| marker != "source dataset identity");
    if !missing_markers
        .iter()
        .any(|marker| marker == "source dataset identity")
    {
        missing_markers.push("source dataset identity".to_string());
    }
}
