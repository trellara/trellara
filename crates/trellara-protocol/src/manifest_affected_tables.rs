use std::collections::BTreeSet;

use crate::{ProtocolError, TransactionManifest};

pub(crate) fn validate_affected_table_event_counts(
    manifest: &TransactionManifest,
) -> Result<(), ProtocolError> {
    if manifest.affected_tables.is_empty() {
        return invalid_manifest_field(
            "affected_tables",
            "must contain at least one affected table",
        );
    }

    let mut seen = BTreeSet::new();
    let affected_event_count =
        manifest
            .affected_tables
            .iter()
            .enumerate()
            .try_fold(0u32, |total, (index, table)| {
                let relation = table.relation.as_ref().ok_or_else(|| {
                    ProtocolError::ManifestAffectedTableMissingRelation {
                        transaction_id: manifest.transaction_id.clone(),
                        index,
                    }
                })?;
                let display_name = relation.display_name();
                if !seen.insert((
                    relation.oid,
                    relation.schema.clone(),
                    relation.table.clone(),
                )) {
                    return Err(ProtocolError::DuplicateManifestAffectedTable {
                        transaction_id: manifest.transaction_id.clone(),
                        relation: display_name,
                    });
                }
                if table.event_count == 0 {
                    return Err(ProtocolError::ManifestAffectedTableEmpty {
                        transaction_id: manifest.transaction_id.clone(),
                        relation: display_name,
                    });
                }
                total.checked_add(table.event_count).ok_or_else(|| {
                    ProtocolError::ManifestCountOverflow {
                        transaction_id: manifest.transaction_id.clone(),
                        field: "affected_tables.event_count",
                        max_supported_count: u32::MAX,
                    }
                })
            })?;

    if affected_event_count != manifest.global_event_count {
        return Err(ProtocolError::ManifestAffectedTableCountMismatch {
            transaction_id: manifest.transaction_id.clone(),
            expected: manifest.global_event_count,
            actual: affected_event_count,
        });
    }

    Ok(())
}

fn invalid_manifest_field<T>(
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, ProtocolError> {
    Err(ProtocolError::InvalidManifestField {
        field,
        reason: reason.into(),
    })
}
