use std::path::Path;

use crate::{
    write_core_pilot_artifacts, write_platform_artifacts, write_support_artifacts,
    PilotPackageArtifact, PilotPackageMaterials, Result, TrellaraConfig,
};

pub(crate) fn write_pilot_package_artifacts(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    correctness_report: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    let mut artifacts = write_core_pilot_artifacts(config, config_path, output, materials)?;
    artifacts.extend(write_platform_artifacts(
        config,
        config_path,
        output,
        materials,
    )?);
    artifacts.extend(write_support_artifacts(
        config,
        config_path,
        output,
        correctness_report,
        materials,
    )?);

    Ok(artifacts)
}
