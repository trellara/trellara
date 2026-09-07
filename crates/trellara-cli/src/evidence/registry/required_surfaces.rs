use crate::EvidenceRegistryReviewSurface;

const REQUIRED_EVIDENCE_REGISTRY_SURFACES: &[&str] = &[
    "source_safety",
    "transaction_boundary",
    "schema_ddl_plan",
    "schema_ddl_envelope_plan",
    "ddl_barrier_status",
    "ddl_release_proof",
    "identity_audit",
    "fleet_evidence_plan",
    "live_evidence_template",
    "control_plane_pull",
    "support_diagnostics",
    "single_review_bundle",
    "lake_writer_plan",
    "lake_fanin_run",
    "lake_completeness",
    "spark_golden_fixture",
];

pub(crate) fn missing_required_review_surface_codes(
    review_surfaces: &[EvidenceRegistryReviewSurface],
) -> Vec<&'static str> {
    REQUIRED_EVIDENCE_REGISTRY_SURFACES
        .iter()
        .copied()
        .filter(|required| {
            !review_surfaces
                .iter()
                .any(|surface| surface.code == *required)
        })
        .collect()
}

pub(crate) fn required_review_surface_count() -> usize {
    REQUIRED_EVIDENCE_REGISTRY_SURFACES.len()
}
