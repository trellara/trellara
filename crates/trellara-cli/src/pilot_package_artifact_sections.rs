use std::path::Path;

use crate::{
    pilot_package_artifacts::write_pilot_package_artifact, render_enterprise_evaluation_text,
    render_pilot_executive_evidence, render_pilot_guide_text, render_pilot_package_readme,
    render_pilot_scorecard_text, render_quickstart_readiness_text, render_quickstart_text,
    write_operator_handoff_artifacts, PilotPackageArtifact, PilotPackageMaterials, Result,
    TrellaraConfig,
};

pub(crate) fn write_core_pilot_artifacts(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    let mut artifacts = vec![
        write_pilot_package_artifact(
            output,
            "README.md",
            "markdown",
            "operator handoff for the design-partner evaluation package",
            &render_pilot_package_readme(config, config_path),
        )?,
        write_pilot_package_artifact(
            output,
            "quickstart-plan.txt",
            "text",
            "no-broker local evaluation command plan",
            &render_quickstart_text(&materials.quickstart_plan),
        )?,
        write_pilot_package_artifact(
            output,
            "quickstart-readiness.txt",
            "text",
            "readiness checks and next commands for the local proof loop",
            &render_quickstart_readiness_text(&materials.quickstart_readiness),
        )?,
        write_pilot_package_artifact(
            output,
            "pilot-guide.txt",
            "text",
            "human-readable deployment phases, proof commands, failure drill, and acceptance gates",
            &render_pilot_guide_text(&materials.pilot_guide),
        )?,
        write_pilot_package_artifact(
            output,
            "pilot-guide.json",
            "json",
            "machine-readable deployment phases and proof evidence",
            &serde_json::to_string_pretty(&materials.pilot_guide)?,
        )?,
        write_pilot_package_artifact(
            output,
            "pilot-scorecard.txt",
            "text",
            "enterprise acceptance scorecard with proof commands and live-evidence gates",
            &render_pilot_scorecard_text(&materials.pilot_scorecard),
        )?,
        write_pilot_package_artifact(
            output,
            "pilot-scorecard.json",
            "json",
            "machine-readable enterprise acceptance scorecard",
            &serde_json::to_string_pretty(&materials.pilot_scorecard)?,
        )?,
        write_pilot_package_artifact(
            output,
            "executive-evidence.md",
            "markdown",
            "executive evidence brief for source safety, transaction-boundary proof, and recovery controls",
            &render_pilot_executive_evidence(&materials.executive_evidence),
        )?,
        write_pilot_package_artifact(
            output,
            "enterprise-evaluation.txt",
            "text",
            "buyer-facing enterprise readiness summary with mode fit, proof commands, and blockers",
            &render_enterprise_evaluation_text(&materials.enterprise_evaluation),
        )?,
        write_pilot_package_artifact(
            output,
            "enterprise-evaluation.json",
            "json",
            "machine-readable enterprise readiness summary with consistency-mode contract",
            &serde_json::to_string_pretty(&materials.enterprise_evaluation)?,
        )?,
        write_pilot_package_artifact(
            output,
            "schema-ddl-plan.json",
            "json",
            "machine-readable policy-gated DDL propagation plan with schema-barrier release gates",
            &serde_json::to_string_pretty(&materials.schema_ddl_plan)?,
        )?,
        write_pilot_package_artifact(
            output,
            "schema-ddl-apply-plan.json",
            "json",
            "dry-run target Postgres DDL apply plan with safe SQL and barrier release sequence",
            &serde_json::to_string_pretty(&materials.schema_ddl_apply_plan)?,
        )?,
        write_pilot_package_artifact(
            output,
            "schema-ddl-envelope-plan.json",
            "json",
            "runtime DDL envelope plan with transaction barrier, event classification, propagation boundary, propagation decisions, policy digest, DML release blockers, and replay-envelope proof with ddl_events stripped",
            &serde_json::to_string_pretty(&materials.schema_ddl_envelope_plan)?,
        )?,
        write_pilot_package_artifact(
            output,
            "ddl-barrier-status.json",
            "json",
            "sample DDL barrier status showing release blockers and stable release_blocker_codes before DML visibility",
            &serde_json::to_string_pretty(&materials.ddl_barrier_status)?,
        )?,
        write_pilot_package_artifact(
            output,
            "ddl-release-proof.json",
            "json",
            "sample DDL release proof showing required sink ACK evidence for post-DDL DML visibility",
            &serde_json::to_string_pretty(&materials.ddl_release_proof)?,
        )?,
    ];
    artifacts.extend(write_operator_handoff_artifacts(
        config,
        config_path,
        output,
        materials,
    )?);
    Ok(artifacts)
}
