use serde::Serialize;
use trellara_checkpoint::SnapshotRunState;

use crate::{
    render_run_summary_text, ApplySummary, BootstrapSummary, QuickstartOutputFormat, RelaySummary,
    Result, RunProofGate, SnapshotCopySummary, VerifySummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RunSummary {
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) stream_kind: String,
    pub(crate) bootstrap: BootstrapSummary,
    pub(crate) apply_schema_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) snapshot: Option<SnapshotCopySummary>,
    pub(crate) relay: RelaySummary,
    pub(crate) apply: ApplySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) verify: Option<VerifySummary>,
    pub(crate) proof_chain: Vec<RunProofGate>,
    pub(crate) next_commands: Vec<String>,
}

pub(crate) fn render_run_summary(
    summary: &RunSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_run_summary_text(summary)),
    }
}

pub(crate) fn mark_snapshot_summary_verified(
    snapshot: Option<&mut SnapshotCopySummary>,
    verify: Option<&VerifySummary>,
) {
    let Some(snapshot) = snapshot else {
        return;
    };
    if verify.is_some_and(|verify| verify.converged) {
        snapshot.state = SnapshotRunState::Verified.to_string();
        snapshot.consistency_note = format!(
            "{}; full-table verification converged and promoted the snapshot run to verified",
            snapshot.consistency_note
        );
    }
}
