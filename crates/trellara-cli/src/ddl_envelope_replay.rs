use serde::Serialize;
use trellara_protocol::TransactionEnvelope;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlEnvelopeReplaySummary {
    pub(crate) barrier_id: Option<String>,
    pub(crate) release_gate: String,
    pub(crate) original_checksum: u64,
    pub(crate) replay_checksum: u64,
    pub(crate) dml_change_count: usize,
    pub(crate) ddl_event_count: usize,
    pub(crate) ddl_events_stripped: bool,
    pub(crate) replay_allowed: bool,
    pub(crate) blocked_until: String,
}

pub(crate) fn ddl_envelope_replay_summary(
    envelope: &TransactionEnvelope,
    barrier_id: Option<&str>,
) -> Option<DdlEnvelopeReplaySummary> {
    if envelope.ddl_events.is_empty() {
        return None;
    }

    let replay = envelope.dml_replay_after_ddl_barrier();
    Some(DdlEnvelopeReplaySummary {
        barrier_id: barrier_id.map(str::to_string),
        release_gate: "post_ddl_dml_release".to_string(),
        original_checksum: envelope.checksum,
        replay_checksum: replay.checksum,
        dml_change_count: replay.changes.len(),
        ddl_event_count: replay.ddl_events.len(),
        ddl_events_stripped: replay.ddl_events.is_empty(),
        replay_allowed: false,
        blocked_until: dml_replay_blocked_until(barrier_id),
    })
}

fn dml_replay_blocked_until(barrier_id: Option<&str>) -> String {
    match barrier_id {
        Some(barrier_id) => {
            format!("ddl-barrier status for {barrier_id} reports post_ddl_dml_release satisfied")
        }
        None => "ddl-barrier status reports post_ddl_dml_release satisfied".to_string(),
    }
}
