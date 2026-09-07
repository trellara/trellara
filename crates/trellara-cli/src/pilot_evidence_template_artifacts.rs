use std::path::Path;

use crate::{
    live_evidence_artifact_name, live_evidence_collection_requirements,
    live_evidence_expected_markers, PilotEvidenceTemplateArtifact, PilotScorecardStatus,
    PilotScorecardSummary,
};

pub(crate) fn pilot_evidence_template_artifacts(
    scorecard: &PilotScorecardSummary,
    output: &Path,
) -> Vec<PilotEvidenceTemplateArtifact> {
    scorecard
        .gates
        .iter()
        .filter(|gate| gate.status == PilotScorecardStatus::NeedsLiveEvidence)
        .map(|gate| PilotEvidenceTemplateArtifact {
            gate_code: gate.code.clone(),
            artifact: output
                .join(live_evidence_artifact_name(&gate.code))
                .display()
                .to_string(),
            proof_command: gate.proof_command.clone(),
            success_markers: live_evidence_expected_markers(&gate.code),
            collection_requirements: live_evidence_collection_requirements(&gate.code),
        })
        .collect::<Vec<_>>()
}
