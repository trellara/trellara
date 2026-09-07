use crate::DdlPlanChangeKind;

pub(crate) fn ddl_change_boundary_rule(kind: DdlPlanChangeKind) -> &'static str {
    match kind {
        DdlPlanChangeKind::ChangePartitionKey => {
            "block partitioned visibility until a fresh snapshot handoff establishes the new partition-key contract"
        }
        DdlPlanChangeKind::ChangePrimaryKey => {
            "block apply until replica identity, target primary-key mapping, and idempotency keys are regenerated"
        }
        DdlPlanChangeKind::AddTable => {
            "start a new table stream only after schema-discover, contract-test, and snapshot handoff pin the table fingerprint"
        }
        _ => {
            "publish a schema barrier before row changes using the new relation fingerprint become visible downstream"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_key_boundary_pauses_partition_visibility_until_handoff() {
        let rule = ddl_change_boundary_rule(DdlPlanChangeKind::ChangePartitionKey);

        assert!(rule.contains("partitioned visibility"));
        assert!(rule.contains("fresh snapshot handoff"));
        assert!(rule.contains("partition-key contract"));
    }

    #[test]
    fn primary_key_boundary_blocks_until_identity_contracts_are_regenerated() {
        let rule = ddl_change_boundary_rule(DdlPlanChangeKind::ChangePrimaryKey);

        assert!(rule.contains("replica identity"));
        assert!(rule.contains("primary-key mapping"));
        assert!(rule.contains("idempotency keys"));
    }

    #[test]
    fn add_table_boundary_requires_contract_test_and_snapshot_handoff() {
        let rule = ddl_change_boundary_rule(DdlPlanChangeKind::AddTable);

        assert!(rule.contains("schema-discover"));
        assert!(rule.contains("contract-test"));
        assert!(rule.contains("snapshot handoff"));
    }

    #[test]
    fn additive_column_boundary_uses_schema_barrier_visibility_rule() {
        let rule = ddl_change_boundary_rule(DdlPlanChangeKind::AddNullableColumn);

        assert!(rule.contains("schema barrier"));
        assert!(rule.contains("row changes"));
        assert!(rule.contains("relation fingerprint"));
    }
}
