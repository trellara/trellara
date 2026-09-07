use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{
    evidence_registry_artifact_path, EvidenceRegistryArtifact, EvidenceRegistryArtifactStatus,
    PilotPackageManifest,
};

pub(crate) struct EvidenceRegistryArtifactCounts {
    pub(crate) verified: usize,
    pub(crate) missing: usize,
    pub(crate) digest_mismatch: usize,
}

pub(crate) fn verify_evidence_registry_artifacts(
    package: &Path,
    manifest: &PilotPackageManifest,
    issues: &mut Vec<String>,
) -> Vec<EvidenceRegistryArtifact> {
    let mut artifacts = Vec::with_capacity(manifest.artifacts.len());
    for artifact in &manifest.artifacts {
        let artifact_path = evidence_registry_artifact_path(package, &artifact.path);
        match fs::read(&artifact_path) {
            Ok(contents) => {
                let actual_sha256 = format!("{:x}", Sha256::digest(&contents));
                let actual_byte_count = contents.len();
                let registry_status = if actual_byte_count == artifact.byte_count
                    && actual_sha256 == artifact.sha256
                {
                    EvidenceRegistryArtifactStatus::Verified
                } else {
                    issues.push(format!(
                            "{} digest mismatch: manifest bytes={} sha256={}, actual bytes={} sha256={}",
                            artifact.path,
                            artifact.byte_count,
                            artifact.sha256,
                            actual_byte_count,
                            actual_sha256
                        ));
                    EvidenceRegistryArtifactStatus::DigestMismatch
                };
                artifacts.push(EvidenceRegistryArtifact {
                    path: artifact.path.clone(),
                    kind: artifact.kind.clone(),
                    purpose: artifact.purpose.clone(),
                    byte_count: actual_byte_count,
                    sha256: actual_sha256,
                    registry_status,
                });
            }
            Err(_) => {
                issues.push(format!(
                    "{} is missing from the proof package",
                    artifact.path
                ));
                artifacts.push(EvidenceRegistryArtifact {
                    path: artifact.path.clone(),
                    kind: artifact.kind.clone(),
                    purpose: artifact.purpose.clone(),
                    byte_count: 0,
                    sha256: String::new(),
                    registry_status: EvidenceRegistryArtifactStatus::Missing,
                });
            }
        }
    }

    artifacts
}

pub(crate) fn evidence_registry_artifact_counts(
    artifacts: &[EvidenceRegistryArtifact],
) -> EvidenceRegistryArtifactCounts {
    EvidenceRegistryArtifactCounts {
        verified: artifacts
            .iter()
            .filter(|artifact| artifact.registry_status == EvidenceRegistryArtifactStatus::Verified)
            .count(),
        missing: artifacts
            .iter()
            .filter(|artifact| artifact.registry_status == EvidenceRegistryArtifactStatus::Missing)
            .count(),
        digest_mismatch: artifacts
            .iter()
            .filter(|artifact| {
                artifact.registry_status == EvidenceRegistryArtifactStatus::DigestMismatch
            })
            .count(),
    }
}
