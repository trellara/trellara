use std::path::Path;

use crate::{
    pilot_package_artifacts::{render_local_run_proof_artifact, write_pilot_package_artifact},
    render_pilot_deployment_guide, render_pilot_feature_pull_list,
    render_pilot_operational_burden_notes, render_pilot_proof_bundle,
    render_source_safety_checklist_artifact, PilotPackageArtifact, PilotPackageMaterials, Result,
    TrellaraConfig,
};

pub(crate) fn write_operator_handoff_artifacts(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    Ok(vec![
        write_pilot_package_artifact(
            output,
            "local-run-proof.md",
            "markdown",
            "human-readable evidence slot for trellara run --local --verify --format text output",
            &render_local_run_proof_artifact(config, config_path),
        )?,
        write_pilot_package_artifact(
            output,
            "source-safety-checklist.md",
            "markdown",
            "read-only source safety diagnostic checklist for design-partner review",
            &render_source_safety_checklist_artifact(config, config_path),
        )?,
        write_pilot_package_artifact(
            output,
            "proof-bundle.md",
            "markdown",
            "single-review proof chain for source safety, transaction boundaries, convergence, and recovery",
            &render_pilot_proof_bundle(
                config,
                config_path,
                &materials.pilot_guide,
                &materials.pilot_scorecard,
            ),
        )?,
        write_pilot_package_artifact(
            output,
            "deployment-guide.md",
            "markdown",
            "partner-specific deployment guide for the first verified replication flow",
            &render_pilot_deployment_guide(config, config_path),
        )?,
        write_pilot_package_artifact(
            output,
            "operational-burden-notes.md",
            "markdown",
            "before/after incident and operational burden notes for design-partner review",
            &render_pilot_operational_burden_notes(config, config_path),
        )?,
        write_pilot_package_artifact(
            output,
            "feature-pull-list.md",
            "markdown",
            "design-partner pull list for control-plane and enterprise expansion decisions",
            &render_pilot_feature_pull_list(config, config_path, &materials.pilot_scorecard),
        )?,
    ])
}
