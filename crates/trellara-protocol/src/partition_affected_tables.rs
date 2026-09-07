use std::collections::BTreeMap;

use crate::{AffectedTable, ChangeRecord, ProtocolError, RelationId};

pub(crate) fn affected_tables_for_changes<'a>(
    transaction_id: &str,
    changes: impl Iterator<Item = &'a ChangeRecord>,
) -> Result<Vec<AffectedTable>, ProtocolError> {
    let mut affected_tables: BTreeMap<(u32, String, String), (RelationId, u32)> = BTreeMap::new();
    for change in changes {
        if let Some(relation) = &change.relation {
            let key = (
                relation.oid,
                relation.schema.clone(),
                relation.table.clone(),
            );
            let (_, count) = affected_tables
                .entry(key)
                .or_insert_with(|| (relation.clone(), 0));
            increment_affected_table_event_count(transaction_id, count)?;
        }
    }

    Ok(affected_tables
        .into_values()
        .map(|(relation, event_count)| AffectedTable {
            relation: Some(relation),
            event_count,
        })
        .collect())
}

fn increment_affected_table_event_count(
    transaction_id: &str,
    count: &mut u32,
) -> Result<(), ProtocolError> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| ProtocolError::ManifestCountOverflow {
            transaction_id: transaction_id.to_string(),
            field: "affected_tables.event_count",
            max_supported_count: u32::MAX,
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChangeRecord, RelationId};

    fn relation() -> RelationId {
        RelationId {
            oid: 42,
            schema: "public".to_string(),
            table: "orders".to_string(),
        }
    }

    #[test]
    fn affected_tables_counts_changes_by_relation() {
        let changes = [
            ChangeRecord {
                relation: Some(relation()),
                total_order: 1,
                ..Default::default()
            },
            ChangeRecord {
                relation: Some(relation()),
                total_order: 2,
                ..Default::default()
            },
            ChangeRecord {
                relation: None,
                total_order: 3,
                ..Default::default()
            },
        ];

        let affected =
            affected_tables_for_changes("tx-affected", changes.iter()).expect("affected tables");

        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].event_count, 2);
        assert_eq!(affected[0].relation.as_ref().expect("relation").oid, 42);
    }

    #[test]
    fn affected_table_event_count_accepts_u32_max() {
        let mut count = u32::MAX - 1;

        increment_affected_table_event_count("tx-affected", &mut count).expect("increment");

        assert_eq!(count, u32::MAX);
    }

    #[test]
    fn affected_table_event_count_fails_before_wraparound() {
        let mut count = u32::MAX;

        assert!(matches!(
            increment_affected_table_event_count("tx-affected", &mut count),
            Err(ProtocolError::ManifestCountOverflow {
                transaction_id,
                field: "affected_tables.event_count",
                max_supported_count: u32::MAX,
            }) if transaction_id == "tx-affected"
        ));
        assert_eq!(count, u32::MAX);
    }
}
