use super::*;
use trellara_protocol::{
    idempotency_key, AffectedTable, ChangeRecord, ColumnValue, ManifestPartition, Operation,
    RelationId, RowImage, StrictEnvelope, TransactionEnvelope,
};

mod tests_commit_planning;
mod tests_consumer_gate;
mod tests_consumer_gate_policy;
mod tests_ddl_ack;
mod tests_derived_views_ddl_ack;
mod tests_epoch_identity;
mod tests_epoch_summary;
mod tests_raw_cdc_writer;

fn relation() -> RelationId {
    RelationId::new(42, "public", "sales")
}

fn affected_tables(event_count: u32) -> Vec<AffectedTable> {
    vec![AffectedTable {
        relation: Some(relation()),
        event_count,
    }]
}

fn row(id: &str, amount: &str, updated_at: &str) -> RowImage {
    RowImage::new(vec![
        ColumnValue::text("id", 23, id, true),
        ColumnValue::text("amount", 25, amount, false),
        ColumnValue::text("updated_at", 25, updated_at, false),
    ])
}

fn change(
    operation: Operation,
    total_order: u32,
    before: Option<RowImage>,
    after: Option<RowImage>,
) -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation()),
        operation: operation as i32,
        replica_identity: 3,
        before,
        after,
        idempotency_key: idempotency_key("source-a", "0/16B6C50", "tx-1", total_order),
    }
}

fn envelope(changes: Vec<ChangeRecord>) -> TransactionEnvelope {
    envelope_for("source-a", "tx-1", "0/16B6C50", changes)
}

fn envelope_for(
    source_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
    mut changes: Vec<ChangeRecord>,
) -> TransactionEnvelope {
    for change in &mut changes {
        change.transaction_id = transaction_id.to_string();
        change.idempotency_key =
            idempotency_key(source_id, commit_lsn, transaction_id, change.total_order);
    }
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes,
    })
}

fn epoch_config(
    required_sources: &[&str],
    straggler_policy: LakeStragglerPolicy,
) -> LakeEpochConfig {
    LakeEpochConfig::new(
        "epoch-2026-08-16T06",
        "retail",
        required_sources.iter().copied(),
        straggler_policy,
    )
}

fn envelope_for_other_dataset(changes: Vec<ChangeRecord>) -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "other".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes,
    });
    envelope.finalize_checksum();
    envelope
}

fn config() -> LakePlanConfig {
    LakePlanConfig::new(vec![
        LakeTableConfig::new("public", "sales", "id").excluding("updated_at")
    ])
}

fn raw_writer_config(source_bucket_count: u32) -> LakeRawCdcWriterConfig {
    LakeRawCdcWriterConfig::new("retail", "epoch-2026-08-16T06", source_bucket_count)
}
