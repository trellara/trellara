use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotPackageSummary {
    pub(crate) output: String,
    pub(crate) config: String,
    pub(crate) artifact_count: usize,
    pub(crate) artifacts: Vec<PilotPackageArtifact>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct PilotPackageArtifact {
    pub(crate) path: String,
    pub(crate) kind: String,
    pub(crate) purpose: String,
    pub(crate) byte_count: usize,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct PilotPackageManifest {
    pub(crate) config: String,
    pub(crate) artifact_count: usize,
    pub(crate) artifacts: Vec<PilotPackageArtifact>,
}
