use std::collections::BTreeMap;

use trellara_protocol::{DdlEvent, RelationId, TransactionEnvelope};

use crate::LakeError;

pub(super) fn validate_ddl_schema_evidence(
    envelope: &TransactionEnvelope,
) -> Result<(), LakeError> {
    for (_, (relation, event)) in final_ddl_events_by_relation(envelope) {
        let Some(schema_version) = envelope
            .schema_versions
            .iter()
            .find(|version| version.relation.as_ref() == Some(&relation))
        else {
            return Err(LakeError::MissingRawCdcDdlSchemaVersion {
                relation: relation.display_name(),
            });
        };

        if schema_version.version != event.schema_fingerprint_after {
            return Err(LakeError::RawCdcDdlSchemaVersionMismatch {
                relation: relation.display_name(),
                expected: event.schema_fingerprint_after,
                actual: schema_version.version,
            });
        }
    }
    Ok(())
}

fn final_ddl_events_by_relation(
    envelope: &TransactionEnvelope,
) -> BTreeMap<(String, String), (RelationId, &DdlEvent)> {
    let mut events: BTreeMap<(String, String), (RelationId, &DdlEvent)> = BTreeMap::new();
    for event in &envelope.ddl_events {
        let Some(relation) = event.relation.clone() else {
            continue;
        };
        let key = (relation.schema.clone(), relation.table.clone());
        match events.get(&key) {
            Some((_, existing)) if existing.total_order > event.total_order => {}
            _ => {
                events.insert(key, (relation, event));
            }
        }
    }
    events
}
