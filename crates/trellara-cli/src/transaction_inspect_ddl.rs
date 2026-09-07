use serde::Serialize;
use trellara_protocol::{DdlEvent, TransactionEnvelope};

use crate::protocol_ddl_operation_label;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectDdlEvent {
    pub(crate) total_order: u32,
    pub(crate) operation: String,
    pub(crate) relation: String,
    pub(crate) target_auto_apply: bool,
    pub(crate) release_gate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectDmlReplay {
    pub(crate) barrier_id: String,
    pub(crate) change_count: usize,
    pub(crate) checksum: u64,
    pub(crate) ddl_events_stripped: bool,
    pub(crate) reason: String,
}

pub(crate) fn transaction_inspect_ddl_events(
    envelope: &TransactionEnvelope,
) -> Vec<TransactionInspectDdlEvent> {
    envelope.ddl_events.iter().map(ddl_event_summary).collect()
}

pub(crate) fn transaction_inspect_dml_replay(
    envelope: &TransactionEnvelope,
) -> Option<TransactionInspectDmlReplay> {
    if envelope.ddl_events.is_empty() {
        return None;
    }
    let replay = envelope.dml_replay_after_ddl_barrier();
    Some(TransactionInspectDmlReplay {
        barrier_id: transaction_inspect_ddl_barrier_id(envelope),
        change_count: replay.changes.len(),
        checksum: replay.checksum,
        ddl_events_stripped: replay.ddl_events.is_empty(),
        reason: "use only after the DDL barrier releases post-DDL DML".to_string(),
    })
}

fn transaction_inspect_ddl_barrier_id(envelope: &TransactionEnvelope) -> String {
    envelope
        .boundary_key()
        .map(|boundary| format!("{boundary}:ddl"))
        .unwrap_or_else(|_| {
            format!(
                "{}:{}:{}:{}:ddl",
                envelope.source_id,
                envelope.dataset_id,
                envelope.transaction_id,
                envelope.commit_lsn
            )
        })
}

fn ddl_event_summary(event: &DdlEvent) -> TransactionInspectDdlEvent {
    TransactionInspectDdlEvent {
        total_order: event.total_order,
        operation: protocol_ddl_operation_label(event).to_string(),
        relation: event
            .relation
            .as_ref()
            .map(|relation| relation.display_name())
            .unwrap_or_else(|| "unknown".to_string()),
        target_auto_apply: event.target_auto_apply,
        release_gate: event.release_gate.clone(),
    }
}
