use std::path::Path;

use crate::{
    pilot_package_artifacts::{write_pilot_package_artifact, write_pilot_package_binary_artifact},
    pilot_package_lake_completeness::LakeCompletenessEvidence,
    pilot_package_lake_writer_sample_envelope, PilotPackageArtifact, PilotPackageMaterials, Result,
    TrellaraConfig,
};

pub(crate) fn write_platform_lake_artifacts(
    config: &TrellaraConfig,
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    let sample_envelope = pilot_package_lake_writer_sample_envelope(config)?.encode_checked()?;

    Ok(vec![
        write_pilot_package_artifact(
            output,
            "lake-ddl.json",
            "json",
            "machine-readable raw CDC and epoch metadata DDL plan with Spark template outputs",
            &serde_json::to_string_pretty(&materials.lake_ddl)?,
        )?,
        write_pilot_package_artifact(
            output,
            "lake-epoch.json",
            "json",
            "machine-readable fleet fan-in epoch completeness decision for data-platform review",
            &serde_json::to_string_pretty(&materials.lake_epoch)?,
        )?,
        write_pilot_package_artifact(
            output,
            "lake-verify.json",
            "json",
            "machine-readable stream-to-lake epoch reconciliation report",
            &serde_json::to_string_pretty(&materials.lake_verify)?,
        )?,
        write_pilot_package_artifact(
            output,
            "lake-completeness.json",
            "json",
            "machine-readable Iceberg completeness evidence tying epoch, verification, committer, and Spark gates together",
            &serde_json::to_string_pretty(&LakeCompletenessEvidence::from_materials(materials))?,
        )?,
        write_pilot_package_artifact(
            output,
            "lake-writer-plan.json",
            "json",
            "machine-readable raw CDC append-file and epoch-metadata writer intent for a verified sample envelope",
            &serde_json::to_string_pretty(&materials.lake_writer_plan)?,
        )?,
        write_pilot_package_binary_artifact(
            output,
            "sample-envelope.pb",
            "protobuf",
            "deterministic transaction envelope used by lake writer-plan and fan-in run package commands",
            &sample_envelope,
        )?,
        write_pilot_package_artifact(
            output,
            "lake-fanin-run.json",
            "json",
            "machine-readable bounded lake fan-in dry-run verdict with replay and Spark release gates",
            &serde_json::to_string_pretty(&materials.lake_fanin_run)?,
        )?,
    ])
}
