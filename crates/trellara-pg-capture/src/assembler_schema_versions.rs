use std::collections::{HashMap, HashSet};

use trellara_protocol::{RelationId, RelationSchemaVersion};

use crate::assembler_pending_ddl::PendingTransactionDdlEvent;
use crate::{CaptureError, Result};

pub(crate) fn record_relation_schema_version(
    relation_schema_versions: &mut HashMap<u32, RelationSchemaVersion>,
    relation: RelationId,
    schema_fingerprint: u64,
) {
    relation_schema_versions.insert(
        relation.oid,
        RelationSchemaVersion {
            relation: Some(relation),
            version: schema_fingerprint,
        },
    );
}

pub(crate) fn schema_versions_for_transaction(
    relation_schema_versions: &HashMap<u32, RelationSchemaVersion>,
    changed_relation_oids: Vec<u32>,
    ddl_events: &[PendingTransactionDdlEvent],
) -> Result<Vec<RelationSchemaVersion>> {
    let mut seen = HashSet::new();
    changed_relation_oids
        .into_iter()
        .filter(|oid| seen.insert(*oid))
        .map(|oid| {
            relation_schema_versions
                .get(&oid)
                .cloned()
                .or_else(|| ddl_schema_version_for_relation(ddl_events, oid))
                .ok_or(CaptureError::MissingRelationSchemaVersion { relation_oid: oid })
        })
        .collect()
}

fn ddl_schema_version_for_relation(
    ddl_events: &[PendingTransactionDdlEvent],
    relation_oid: u32,
) -> Option<RelationSchemaVersion> {
    ddl_events.iter().rev().find_map(|pending| {
        let relation = pending.event.relation.as_ref()?;
        if relation.oid == relation_oid && pending.event.schema_fingerprint_after > 0 {
            Some(RelationSchemaVersion {
                relation: Some(relation.clone()),
                version: pending.event.schema_fingerprint_after,
            })
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_protocol::RelationId;

    #[test]
    fn schema_versions_follow_first_changed_relation_order_without_duplicates() {
        let mut versions = HashMap::new();
        versions.insert(10, schema_version(10, "sales", 1));
        versions.insert(20, schema_version(20, "refunds", 2));

        let selected = schema_versions_for_transaction(&versions, vec![20, 10, 20, 10], &[])
            .expect("versions");

        assert_eq!(
            selected
                .iter()
                .map(|version| version.version)
                .collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    #[test]
    fn schema_versions_reject_missing_changed_relation_metadata() {
        let mut versions = HashMap::new();
        versions.insert(10, schema_version(10, "sales", 1));

        let error = schema_versions_for_transaction(&versions, vec![10, 20], &[])
            .expect_err("missing relation metadata rejected");

        assert!(matches!(
            error,
            CaptureError::MissingRelationSchemaVersion { relation_oid: 20 }
        ));
    }

    fn schema_version(oid: u32, table: &str, version: u64) -> RelationSchemaVersion {
        RelationSchemaVersion {
            relation: Some(RelationId::new(oid, "public", table)),
            version,
        }
    }
}
