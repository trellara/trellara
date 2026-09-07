use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetLakeFaninReadiness {
    pub(crate) status: FleetLakeFaninStatus,
    pub(crate) fanin_mode: String,
    pub(crate) epoch_visibility_boundary: String,
    pub(crate) iceberg_checkpoint_receipt_gate: String,
    pub(crate) straggler_policy: String,
    pub(crate) source_watermark_rollup: String,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) guidance: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FleetLakeFaninStatus {
    Ready,
    PublishableWithGaps,
    Blocked,
}

impl FleetLakeFaninStatus {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::PublishableWithGaps => "publishable_with_gaps",
            Self::Blocked => "blocked",
        }
    }
}
