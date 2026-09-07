use super::*;
use trellara_protocol::{ColumnValue, Operation, ReplicaIdentity, RowImage};

#[test]
fn assembler_drops_aborted_stream_subtransaction_changes() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");
    record_sales_relation_metadata(&mut assembler, &config);

    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: Some("43".to_string()),
                relation: relation.clone(),
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id",
                    23,
                    "rolled-back",
                    true,
                )])),
            },
        )
        .expect("subtransaction change");
    assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: Some("42".to_string()),
                relation,
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id",
                    23,
                    "committed",
                    true,
                )])),
            },
        )
        .expect("parent change");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");
    assert!(assembler
        .streamed
        .get("42")
        .expect("streamed transaction")
        .changes
        .is_spilled());
    assembler
        .apply(
            &config,
            LogicalEvent::StreamAbort {
                transaction_id: "42".to_string(),
                subtransaction_id: "43".to_string(),
            },
        )
        .expect("subtransaction abort");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("stream commit")
        .expect("streamed envelope");

    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(envelope.changes[0].total_order, 1);
    assert_eq!(envelope.changes[0].table_order, 1);
    assert_eq!(envelope.changes[0].partition_order, 1);
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6C50:42:1"
    );
    assert_eq!(
        envelope.changes[0].after.as_ref().expect("after").columns[0].text_value,
        "committed"
    );
}
