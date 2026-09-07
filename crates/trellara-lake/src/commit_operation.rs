use trellara_protocol::{ChangeRecord, RelationId, TransactionEnvelope};

use crate::{LakeColumnValue, LakeMaterialization, LakeWriteKind, LakeWriteOperation};

pub(crate) fn commit_operation(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
    change: &ChangeRecord,
    materialization: LakeMaterialization,
    write_kind: LakeWriteKind,
    record_key: Option<String>,
    row: Vec<LakeColumnValue>,
) -> LakeWriteOperation {
    LakeWriteOperation {
        materialization,
        write_kind,
        relation: relation.display_name(),
        total_order: change.total_order,
        record_key,
        source_transaction_id: envelope.transaction_id.clone(),
        source_commit_lsn: envelope.commit_lsn.clone(),
        source_commit_timestamp_ms: envelope.commit_timestamp_ms,
        row,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_protocol::{
        ChangeRecord, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    };

    #[test]
    fn commit_operation_preserves_transaction_boundary_metadata() {
        let envelope = TransactionEnvelope::strict(StrictEnvelope {
            source_id: "store-001".to_string(),
            database_id: "postgres".to_string(),
            dataset_id: "retail".to_string(),
            transaction_id: "tx-1".to_string(),
            begin_lsn: "0/16B6C00".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 1_786_420_000_000,
            changes: vec![change_record()],
        });
        let relation = RelationId::new(16_384, "public", "sales");

        let operation = commit_operation(
            &envelope,
            &relation,
            &envelope.changes[0],
            LakeMaterialization::RawCdc,
            LakeWriteKind::AppendEvent,
            Some("sale-1".to_string()),
            Vec::new(),
        );

        assert_eq!(operation.relation, "public.sales");
        assert_eq!(operation.total_order, 1);
        assert_eq!(operation.record_key.as_deref(), Some("sale-1"));
        assert_eq!(operation.source_transaction_id, "tx-1");
        assert_eq!(operation.source_commit_lsn, "0/16B6C50");
        assert_eq!(operation.source_commit_timestamp_ms, 1_786_420_000_000);
    }

    fn change_record() -> ChangeRecord {
        ChangeRecord {
            transaction_id: "tx-1".to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(RelationId::new(16_384, "public", "sales")),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(RowImage::new(Vec::new())),
            idempotency_key: "store-001:0/16B6C50:tx-1:1".to_string(),
        }
    }
}
