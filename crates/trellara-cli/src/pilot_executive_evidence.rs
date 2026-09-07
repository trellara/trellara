use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PilotExecutiveEvidenceSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) north_star: String,
    pub(crate) evaluation_thesis: String,
    pub(crate) transaction_boundary: String,
    pub(crate) stream_kind: String,
    pub(crate) kafka_required: bool,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) differentiated_controls: Vec<String>,
    pub(crate) live_evidence_required: Vec<String>,
    pub(crate) executive_decision: String,
}
