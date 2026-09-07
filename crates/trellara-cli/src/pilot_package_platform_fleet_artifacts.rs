use std::path::Path;

use crate::{
    pilot_package_artifacts::write_pilot_package_artifact, render_fleet_evidence_plan_text,
    render_fleet_report_text, render_fleet_scorecard_text, PilotPackageArtifact,
    PilotPackageMaterials, Result,
};

pub(crate) fn write_platform_fleet_artifacts(
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    let mut artifacts = vec![
        write_pilot_package_artifact(
            output,
            "fleet-report.txt",
            "text",
            "fleet topology, convergence-gate, and recovery-drill report for design-partner review",
            &render_fleet_report_text(&materials.fleet_report),
        )?,
        write_pilot_package_artifact(
            output,
            "fleet-scorecard.txt",
            "text",
            "fleet readiness go/no-go scorecard for design-partner review",
            &render_fleet_scorecard_text(&materials.fleet_scorecard),
        )?,
        write_pilot_package_artifact(
            output,
            "fleet-evidence-plan.txt",
            "text",
            "per-flow live-evidence command plan for design-partner review",
            &render_fleet_evidence_plan_text(&materials.fleet_evidence_plan),
        )?,
        write_pilot_package_artifact(
            output,
            "consistency-contract.json",
            "json",
            "machine-readable flow-level consistency contract for transaction visibility and checkpoint advancement",
            &serde_json::to_string_pretty(&materials.consistency_contract)?,
        )?,
        write_pilot_package_artifact(
            output,
            "performance-envelope.json",
            "json",
            "machine-readable configuration-derived performance envelope for pilot throughput and durability review",
            &serde_json::to_string_pretty(&materials.performance_envelope)?,
        )?,
        write_pilot_package_artifact(
            output,
            "identity-audit.json",
            "json",
            "machine-readable primary-key, replica-identity, and TOAST apply audit",
            &serde_json::to_string_pretty(&materials.identity_audit)?,
        )?,
        write_pilot_package_artifact(
            output,
            "consumer-semantics.json",
            "json",
            "machine-readable consumer semantics matrix with strict and partitioned transaction-boundary language",
            &serde_json::to_string_pretty(&materials.consumer_semantics)?,
        )?,
        write_pilot_package_artifact(
            output,
            "fleet-control-plane.json",
            "json",
            "conservative hosted control-plane pull report grounded in local fleet evidence",
            &serde_json::to_string_pretty(&materials.fleet_control_plane)?,
        )?,
    ];

    if let Some(plan) = &materials.partition_rebalance_plan {
        artifacts.push(write_pilot_package_artifact(
            output,
            "partition-rebalance-plan.json",
            "json",
            "machine-readable partitioned scale rebalance governance plan with runtime movement disabled",
            &serde_json::to_string_pretty(plan)?,
        )?);
    }

    Ok(artifacts)
}
