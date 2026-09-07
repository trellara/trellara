pub(crate) mod artifacts;
pub(crate) mod render;
pub(crate) mod required_surfaces;
pub(crate) mod support;
pub(crate) mod surface_specs;
pub(crate) mod surfaces;

use std::fs;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    evidence_registry_artifact_counts, evidence_registry_correctness_report,
    evidence_registry_review_surfaces, missing_required_review_surface_codes,
    required_review_surface_count, verify_evidence_registry_artifacts, CliError,
    PilotPackageManifest, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EvidenceRegistrySummary {
    pub(crate) package: String,
    pub(crate) manifest: String,
    pub(crate) config: String,
    pub(crate) verified: bool,
    pub(crate) artifact_count: usize,
    pub(crate) verified_artifact_count: usize,
    pub(crate) missing_artifact_count: usize,
    pub(crate) digest_mismatch_count: usize,
    pub(crate) package_manifest_sha256: String,
    pub(crate) correctness_report: Option<EvidenceRegistryCorrectnessReport>,
    pub(crate) review_surface_count: usize,
    pub(crate) required_review_surface_count: usize,
    pub(crate) missing_required_review_surface_count: usize,
    pub(crate) artifacts: Vec<EvidenceRegistryArtifact>,
    pub(crate) review_surfaces: Vec<EvidenceRegistryReviewSurface>,
    pub(crate) issues: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EvidenceRegistryArtifact {
    pub(crate) path: String,
    pub(crate) kind: String,
    pub(crate) purpose: String,
    pub(crate) byte_count: usize,
    pub(crate) sha256: String,
    pub(crate) registry_status: EvidenceRegistryArtifactStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvidenceRegistryArtifactStatus {
    Verified,
    Missing,
    DigestMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EvidenceRegistryCorrectnessReport {
    pub(crate) path: String,
    pub(crate) present: bool,
    pub(crate) byte_count: usize,
    pub(crate) sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EvidenceRegistryReviewSurface {
    pub(crate) code: String,
    pub(crate) artifact: String,
    pub(crate) evidence: String,
}

impl EvidenceRegistrySummary {
    pub(crate) fn from_package(package: &Path, correctness_report: Option<&Path>) -> Result<Self> {
        let manifest_path = package.join("manifest.json");
        let manifest_contents =
            fs::read_to_string(&manifest_path).map_err(|source| CliError::ReadInput {
                path: manifest_path.display().to_string(),
                source,
            })?;
        let manifest: PilotPackageManifest = serde_json::from_str(&manifest_contents)?;
        let config_path = manifest.config.clone();
        let package_manifest_sha256 = format!("{:x}", Sha256::digest(manifest_contents.as_bytes()));

        let mut issues = Vec::new();
        let artifacts = verify_evidence_registry_artifacts(package, &manifest, &mut issues);

        if manifest.artifact_count != manifest.artifacts.len() {
            issues.push(format!(
                "manifest artifact_count={} but listed artifacts={}",
                manifest.artifact_count,
                manifest.artifacts.len()
            ));
        }

        let artifact_counts = evidence_registry_artifact_counts(&artifacts);
        let correctness_report = correctness_report.map(evidence_registry_correctness_report);
        if let Some(report) = &correctness_report {
            if !report.present {
                issues.push(format!("correctness report {} is missing", report.path));
            }
        }

        let review_surfaces = evidence_registry_review_surfaces(&artifacts);
        let missing_required_surfaces = missing_required_review_surface_codes(&review_surfaces);
        let required_review_surface_count = required_review_surface_count();
        for code in &missing_required_surfaces {
            issues.push(format!(
                "required review surface {code} is missing or unverified"
            ));
        }
        let verified = issues.is_empty()
            && artifact_counts.verified == manifest.artifacts.len()
            && missing_required_surfaces.is_empty();

        Ok(Self {
            package: package.display().to_string(),
            manifest: manifest_path.display().to_string(),
            config: config_path.clone(),
            verified,
            artifact_count: artifacts.len(),
            verified_artifact_count: artifact_counts.verified,
            missing_artifact_count: artifact_counts.missing,
            digest_mismatch_count: artifact_counts.digest_mismatch,
            package_manifest_sha256,
            correctness_report,
            review_surface_count: review_surfaces.len(),
            required_review_surface_count,
            missing_required_review_surface_count: missing_required_surfaces.len(),
            artifacts,
            review_surfaces,
            issues,
            next_commands: vec![
                format!(
                    "trellara evidence-registry --package {} --format text",
                    package.display()
                ),
                format!(
                    "trellara pilot-package --config {} --output {}",
                    config_path,
                    package.display()
                ),
                "make verify-correctness-report".to_string(),
                "share this registry with enterprise reviewers before building hosted evidence workflows".to_string(),
            ],
        })
    }
}
