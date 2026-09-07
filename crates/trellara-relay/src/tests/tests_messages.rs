use crate::messages::messages_for_envelope;
use crate::RelayMode;
use trellara_protocol::{
    ChangeRecord, ColumnValue, Operation, PartitionKeyChangePolicy, PartitionNullKeyPolicy,
    PartitionPlanConfig, RelationId, ReplicaIdentity, RowImage, StrictChunkPlanConfig,
    StrictEnvelope, TransactionEnvelope,
};
use trellara_stream::StreamHeader;

#[test]
fn strict_mode_plans_single_transaction_message() {
    let messages = messages_for_envelope(&envelope("tx-1", "0/16B6C50"), &RelayMode::Strict)
        .expect("strict messages");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].topic, "trellara.source.sales.strict");
    assert!(messages[0]
        .headers
        .contains(&StreamHeader::new("trellara.transaction_id", "tx-1")));
}

#[test]
fn strict_chunked_mode_orders_chunks_manifest_then_commit() {
    let messages = messages_for_envelope(
        &multi_change_envelope("tx-large", "0/16B6C50"),
        &RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        }),
    )
    .expect("strict chunked messages");

    assert_eq!(messages.len(), 5);
    assert!(messages[..3]
        .iter()
        .all(|message| message.topic == "trellara.source.sales.strict"));
    assert_eq!(messages[3].topic, "trellara.source.sales.manifest");
    assert_eq!(messages[4].topic, "trellara.source.sales.commit");
}

#[test]
fn partitioned_mode_orders_partition_chunks_manifest_then_commit() {
    let messages = messages_for_envelope(
        &envelope("tx-1", "0/16B6C50"),
        &RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        }),
    )
    .expect("partitioned messages");

    assert_eq!(messages.len(), 3);
    assert!(messages[0]
        .topic
        .starts_with("trellara.source.sales.partition."));
    assert_eq!(messages[1].topic, "trellara.source.sales.manifest");
    assert_eq!(messages[2].topic, "trellara.source.sales.commit");
}

fn envelope(transaction_id: &str, commit_lsn: &str) -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![change(transaction_id, commit_lsn, 1)],
    })
}

fn multi_change_envelope(transaction_id: &str, commit_lsn: &str) -> TransactionEnvelope {
    let mut envelope = envelope(transaction_id, commit_lsn);
    envelope.changes = (1..=5)
        .map(|order| change(transaction_id, commit_lsn, order))
        .collect();
    envelope.finalize_checksum();
    envelope
}

fn change(transaction_id: &str, commit_lsn: &str, order: u32) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order: order,
        table_order: order,
        partition_order: order,
        relation: Some(RelationId::new(42, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![ColumnValue::text(
            "store_id",
            25,
            format!("store-{order}"),
            true,
        )])),
        idempotency_key: format!("source:{commit_lsn}:{transaction_id}:{order}"),
    }
}
