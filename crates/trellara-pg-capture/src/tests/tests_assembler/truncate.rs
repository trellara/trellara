use super::*;
use trellara_protocol::Operation;

#[test]
fn assembler_expands_truncate_relations_in_source_order() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let sales = RelationId::new(16_384, "public", "sales");
    let sale_items = RelationId::new(16_385, "public", "sale_items");

    for (relation, schema_fingerprint) in [(sales.clone(), 12_345), (sale_items.clone(), 67_890)] {
        assembler
            .apply(
                &config,
                LogicalEvent::RelationMetadata {
                    relation,
                    schema_fingerprint,
                },
            )
            .expect("relation metadata");
    }

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-truncate".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::Truncate {
                transaction_id: None,
                relations: vec![sales, sale_items],
            },
        )
        .expect("truncate");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("envelope");

    assert_eq!(envelope.changes.len(), 2);
    assert_eq!(envelope.changes[0].operation, Operation::Truncate as i32);
    assert_eq!(envelope.changes[0].total_order, 1);
    assert_eq!(
        envelope.changes[0]
            .relation
            .as_ref()
            .expect("relation")
            .table,
        "sales"
    );
    assert_eq!(envelope.changes[1].total_order, 2);
    assert_eq!(
        envelope.changes[1]
            .relation
            .as_ref()
            .expect("relation")
            .table,
        "sale_items"
    );
    assert!(envelope.changes.iter().all(|change| {
        change.before.is_none() && change.after.is_none() && !change.idempotency_key.is_empty()
    }));
    envelope.verify_checksum().expect("checksum");
}
