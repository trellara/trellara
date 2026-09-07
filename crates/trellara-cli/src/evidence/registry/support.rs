use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::EvidenceRegistryCorrectnessReport;

pub(crate) fn evidence_registry_artifact_path(package: &Path, manifest_path: &str) -> PathBuf {
    let path = PathBuf::from(manifest_path);
    if path.exists() {
        return path;
    }
    match path.file_name() {
        Some(file_name) => package.join(file_name),
        None => package.join(manifest_path),
    }
}

pub(crate) fn evidence_registry_correctness_report(
    path: &Path,
) -> EvidenceRegistryCorrectnessReport {
    match fs::read(path) {
        Ok(contents) => EvidenceRegistryCorrectnessReport {
            path: path.display().to_string(),
            present: true,
            byte_count: contents.len(),
            sha256: Some(format!("{:x}", Sha256::digest(&contents))),
        },
        Err(_) => EvidenceRegistryCorrectnessReport {
            path: path.display().to_string(),
            present: false,
            byte_count: 0,
            sha256: None,
        },
    }
}
