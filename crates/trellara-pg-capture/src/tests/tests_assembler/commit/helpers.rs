use super::*;

pub(super) fn begin_and_insert(
    assembler: &mut TransactionAssembler,
    config: &TransactionAssemblerConfig,
    transaction_id: &str,
    begin_lsn: &str,
) {
    let relation = RelationId::new(16_384, "public", "sales");
    assembler
        .apply(
            config,
            LogicalEvent::RelationMetadata {
                relation: relation.clone(),
                schema_fingerprint: 12_345,
            },
        )
        .expect("relation metadata");
    assembler
        .apply(
            config,
            LogicalEvent::Begin {
                transaction_id: transaction_id.to_string(),
                begin_lsn: begin_lsn.to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            config,
            LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-1", true,
                )])),
            },
        )
        .expect("change");
}
