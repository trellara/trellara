use std::fs;
use std::path::Path;

use crate::{
    pilot_evidence_template_artifacts, pilot_package_artifacts::write_pilot_package_artifact,
    render_pilot_evidence_collect_script, render_pilot_evidence_template_readme, CliError,
    PilotPackageArtifact, PilotPackageMaterials, Result, TrellaraConfig,
};

pub(crate) fn write_support_artifacts(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    correctness_report: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    Ok(vec![
        write_pilot_package_artifact(
            output,
            "correctness-report.html",
            "html",
            "static deterministic correctness report for failure-matrix and replay-boundary review",
            &fs::read_to_string(correctness_report).map_err(|source| CliError::ReadInput {
                path: correctness_report.display().to_string(),
                source,
            })?,
        )?,
        write_pilot_package_artifact(
            output,
            "live-evidence/README.md",
            "markdown",
            "operator handoff for collecting live scorecard evidence",
            &render_pilot_evidence_template_readme(
                config,
                config_path,
                &output.join("live-evidence"),
                &pilot_evidence_template_artifacts(
                    &materials.pilot_scorecard,
                    &output.join("live-evidence"),
                ),
            ),
        )?,
        write_pilot_package_artifact(
            output,
            "live-evidence/collect.sh",
            "shell",
            "reviewable shell script that writes gate evidence artifacts",
            &render_pilot_evidence_collect_script(
                config,
                config_path,
                &output.join("live-evidence"),
                &pilot_evidence_template_artifacts(
                    &materials.pilot_scorecard,
                    &output.join("live-evidence"),
                ),
            ),
        )?,
    ])
}
