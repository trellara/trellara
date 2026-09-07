use std::path::Path;

use crate::{
    pilot_package_artifacts::write_pilot_package_artifact, render_status_view,
    PilotPackageArtifact, PilotPackageMaterials, QuickstartOutputFormat, Result, StatusView,
};

pub(crate) fn write_platform_diagnostics_artifacts(
    config_path: &Path,
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    Ok(vec![
        write_pilot_package_artifact(
            output,
            "diagnostics.txt",
            "text",
            "support-ready diagnostics bundle with report, alerts, repair plan, metrics, latest failure, and attachment commands",
            &render_status_view(
                materials.diagnostics_status.clone(),
                StatusView::Diagnostics,
                QuickstartOutputFormat::Text,
                config_path,
            )?,
        )?,
        write_pilot_package_artifact(
            output,
            "diagnostics.json",
            "json",
            "machine-readable diagnostics bundle for support escalation and control-plane ingestion",
            &render_status_view(
                materials.diagnostics_status.clone(),
                StatusView::Diagnostics,
                QuickstartOutputFormat::Json,
                config_path,
            )?,
        )?,
    ])
}
