use trellara_protocol::{DdlEvent, DdlOperation, RelationId};

pub(crate) struct PendingDdlEvent {
    pub(crate) operation: DdlOperation,
    pub(crate) relation: RelationId,
    pub(crate) statement: String,
    pub(crate) schema_fingerprint_before: u64,
    pub(crate) schema_fingerprint_after: u64,
    pub(crate) target_auto_apply: bool,
    pub(crate) release_gate: String,
}

impl PendingDdlEvent {
    pub(crate) fn into_protocol_event(self) -> DdlEvent {
        DdlEvent {
            transaction_id: String::new(),
            total_order: 0,
            operation: self.operation as i32,
            relation: Some(self.relation),
            statement: self.statement,
            schema_fingerprint_before: self.schema_fingerprint_before,
            schema_fingerprint_after: self.schema_fingerprint_after,
            target_auto_apply: self.target_auto_apply,
            release_gate: self.release_gate,
        }
    }
}
