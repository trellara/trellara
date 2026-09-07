use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierSinkEvidence {
    pub sink: String,
    pub status: String,
    pub ack_lsn: Option<String>,
    pub schema_version: Option<String>,
    pub accepted: Option<bool>,
    pub detail: Option<String>,
    pub release_eligible: bool,
    pub rejection_code: Option<String>,
    pub rejection_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierReleaseGate {
    pub name: String,
    pub satisfied: bool,
    pub evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierReleaseAction {
    pub code: String,
    pub sinks: Vec<String>,
    pub command: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierReleaseBlocker {
    pub code: String,
    pub message: String,
    pub sinks: Vec<String>,
    #[serde(default)]
    pub evidence: String,
}
