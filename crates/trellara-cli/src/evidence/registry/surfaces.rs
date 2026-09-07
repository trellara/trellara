use crate::{
    EvidenceRegistryArtifact, EvidenceRegistryArtifactStatus, EvidenceRegistryReviewSurface,
    EvidenceRegistrySurfaceSpec, EVIDENCE_REGISTRY_SURFACE_SPECS,
};

pub(crate) fn evidence_registry_review_surfaces(
    artifacts: &[EvidenceRegistryArtifact],
) -> Vec<EvidenceRegistryReviewSurface> {
    EVIDENCE_REGISTRY_SURFACE_SPECS
        .iter()
        .filter_map(|spec| verified_surface_for_spec(artifacts, spec))
        .collect()
}
fn verified_surface_for_spec(
    artifacts: &[EvidenceRegistryArtifact],
    spec: &EvidenceRegistrySurfaceSpec,
) -> Option<EvidenceRegistryReviewSurface> {
    artifacts
        .iter()
        .find(|artifact| artifact_matches_surface_spec(artifact, spec))
        .map(|artifact| EvidenceRegistryReviewSurface {
            code: spec.code.to_string(),
            artifact: artifact.path.clone(),
            evidence: spec.evidence.to_string(),
        })
}
fn artifact_matches_surface_spec(
    artifact: &EvidenceRegistryArtifact,
    spec: &EvidenceRegistrySurfaceSpec,
) -> bool {
    artifact.path.ends_with(spec.artifact_name)
        && artifact.registry_status == EvidenceRegistryArtifactStatus::Verified
}
