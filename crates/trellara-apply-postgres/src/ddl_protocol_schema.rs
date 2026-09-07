use trellara_protocol::{DdlEvent, RelationId, TransactionEnvelope};

use crate::{ApplyError, Result};

pub(crate) fn final_ddl_event(envelope: &TransactionEnvelope) -> Result<&DdlEvent> {
    envelope
        .ddl_events
        .iter()
        .max_by_key(|event| event.total_order)
        .ok_or(ApplyError::MissingDdlField {
            field: "schema_version",
        })
}

pub(crate) fn ddl_schema_version(event: &DdlEvent) -> String {
    format!("schema-fingerprint:{}", event.schema_fingerprint_after)
}

pub(crate) fn validate_final_ddl_schema_version_evidence(
    envelope: &TransactionEnvelope,
    final_ddl_event: &DdlEvent,
) -> Result<()> {
    let Some(final_relation) = final_ddl_event.relation.as_ref() else {
        return Ok(());
    };
    let Some(schema_version) = envelope
        .schema_versions
        .iter()
        .find(|schema_version| same_relation(schema_version.relation.as_ref(), final_relation))
    else {
        return Ok(());
    };

    if schema_version.version != final_ddl_event.schema_fingerprint_after {
        return Err(ApplyError::DdlSchemaVersionMismatch {
            relation: final_relation.display_name(),
            expected: final_ddl_event.schema_fingerprint_after,
            actual: schema_version.version,
        });
    }

    Ok(())
}

fn same_relation(candidate: Option<&RelationId>, expected: &RelationId) -> bool {
    candidate.is_some_and(|candidate| {
        candidate.oid == expected.oid
            && candidate.schema == expected.schema
            && candidate.table == expected.table
    })
}
