use std::fs;
use std::path::Path;

use crate::{
    pilot_package_artifact_list::write_pilot_package_artifacts,
    pilot_package_artifacts::write_pilot_package_artifact, pilot_package_next_commands, CliError,
    PilotPackageManifest, PilotPackageMaterials, PilotPackageSummary, Result, TrellaraConfig,
};

pub(crate) fn write_pilot_package(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    correctness_report: &Path,
) -> Result<PilotPackageSummary> {
    fs::create_dir_all(output).map_err(|source| CliError::WriteOutput {
        path: output.display().to_string(),
        source,
    })?;

    let materials = PilotPackageMaterials::from_config(config, config_path)?;
    let mut artifacts =
        write_pilot_package_artifacts(config, config_path, output, correctness_report, &materials)?;

    let manifest = PilotPackageManifest {
        config: config_path.display().to_string(),
        artifact_count: artifacts.len(),
        artifacts: artifacts.clone(),
    };
    artifacts.push(write_pilot_package_artifact(
        output,
        "manifest.json",
        "json",
        "artifact integrity manifest with byte counts and SHA-256 digests",
        &serde_json::to_string_pretty(&manifest)?,
    )?);

    Ok(PilotPackageSummary {
        output: output.display().to_string(),
        config: config_path.display().to_string(),
        artifact_count: artifacts.len(),
        artifacts,
        next_commands: pilot_package_next_commands(config_path, output),
    })
}
