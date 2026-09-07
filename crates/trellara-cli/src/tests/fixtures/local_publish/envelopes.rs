use super::*;
use trellara_protocol::{ChangeRecord, ColumnValue, RelationId, RowImage, StrictEnvelope};

pub(in crate::tests) fn unique_test_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos()
}

pub(super) fn new_local_publisher(config: &TrellaraConfig) -> LocalPublisher {
    LocalPublisher::new(
        config
            .to_local_publisher_config()
            .expect("local publisher config"),
    )
    .expect("local publisher")
}

pub(super) fn local_two_change_envelope(
    config: &TrellaraConfig,
    transaction_id: &str,
    commit_lsn: &str,
) -> TransactionEnvelope {
    let changes = vec![
        local_insert_change(transaction_id, commit_lsn, 1, "sale-1", "store-west"),
        local_insert_change(transaction_id, commit_lsn, 2, "sale-2", "store-east"),
    ];
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: config.source.id.clone(),
        database_id: config
            .source
            .database_id
            .clone()
            .unwrap_or_else(|| "postgres".to_string()),
        dataset_id: config.dataset.id.clone(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6C00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    })
}

fn local_insert_change(
    transaction_id: &str,
    commit_lsn: &str,
    order: u32,
    sale_id: &str,
    store_id: &str,
) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order: order,
        table_order: order,
        partition_order: order,
        relation: Some(RelationId::new(1, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![
            ColumnValue::text("id", 23, sale_id, true),
            ColumnValue::text("store_id", 25, store_id, false),
        ])),
        idempotency_key: format!("local-source:{commit_lsn}:{transaction_id}:{order}"),
    }
}
