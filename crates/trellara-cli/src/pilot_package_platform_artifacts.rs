use std::path::Path;

use crate::{
    write_platform_diagnostics_artifacts, write_platform_fleet_artifacts,
    write_platform_lake_artifacts, write_platform_spark_artifacts, PilotPackageArtifact,
    PilotPackageMaterials, Result, TrellaraConfig,
};

pub(crate) fn write_platform_artifacts(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    let mut artifacts = write_platform_fleet_artifacts(output, materials)?;
    artifacts.extend(write_platform_lake_artifacts(config, output, materials)?);
    artifacts.extend(write_platform_spark_artifacts(output, materials)?);
    artifacts.extend(write_platform_diagnostics_artifacts(
        config_path,
        output,
        materials,
    )?);

    Ok(artifacts)
}
