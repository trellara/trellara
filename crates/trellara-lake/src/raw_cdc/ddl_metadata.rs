use trellara_protocol::{DdlEvent, RelationId, TransactionEnvelope};

pub(super) struct RawCdcDdlMetadata {
    pub(super) schema_version: Option<u64>,
    pub(super) ddl_barrier_id: Option<String>,
    pub(super) ddl_release_gate: Option<String>,
    pub(super) schema_fingerprint_before: Option<u64>,
    pub(super) schema_fingerprint_after: Option<u64>,
}

pub(super) fn ddl_metadata(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
) -> RawCdcDdlMetadata {
    let ddl_event = ddl_event(envelope, relation);
    RawCdcDdlMetadata {
        schema_version: schema_version(envelope, relation),
        ddl_barrier_id: ddl_event.map(|_| ddl_barrier_id(envelope)),
        ddl_release_gate: ddl_event.map(|event| event.release_gate.clone()),
        schema_fingerprint_before: ddl_event.map(|event| event.schema_fingerprint_before),
        schema_fingerprint_after: ddl_event.map(|event| event.schema_fingerprint_after),
    }
}

fn schema_version(envelope: &TransactionEnvelope, relation: &RelationId) -> Option<u64> {
    envelope
        .schema_versions
        .iter()
        .find(|version| {
            version
                .relation
                .as_ref()
                .is_some_and(|item| item == relation)
        })
        .map(|version| version.version)
}

fn ddl_event<'a>(envelope: &'a TransactionEnvelope, relation: &RelationId) -> Option<&'a DdlEvent> {
    envelope
        .ddl_events
        .iter()
        .filter(|event| event.relation.as_ref().is_some_and(|item| item == relation))
        .max_by_key(|event| event.total_order)
}

fn ddl_barrier_id(envelope: &TransactionEnvelope) -> String {
    format!(
        "{}:{}:{}:{}:ddl",
        envelope.source_id, envelope.dataset_id, envelope.transaction_id, envelope.commit_lsn
    )
}
