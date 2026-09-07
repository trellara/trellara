use trellara_protocol::{
    idempotency_key, ChangeRecord, RelationSchemaVersion, StrictEnvelope, TransactionEnvelope,
};

use crate::assembler_config::TransactionAssemblerConfig;
use crate::assembler_order::{compact_event_orders, compacted_order};
use crate::assembler_transaction::OpenTransaction;
use crate::Result;

pub(crate) fn transaction_into_envelope(
    transaction: OpenTransaction,
    config: &TransactionAssemblerConfig,
    commit_lsn: String,
    commit_timestamp_ms: i64,
    schema_versions: Vec<RelationSchemaVersion>,
) -> Result<TransactionEnvelope> {
    let pending_changes = transaction.changes.into_changes()?;
    let order_map = compact_event_orders(
        &transaction.transaction_id,
        &pending_changes,
        &transaction.ddl_events,
    )?;
    let changes = pending_changes
        .into_iter()
        .map(|pending| {
            let total_order = compacted_order(&order_map, pending.total_order)?;
            Ok(ChangeRecord {
                transaction_id: transaction.transaction_id.clone(),
                total_order,
                table_order: total_order,
                partition_order: total_order,
                relation: Some(pending.relation),
                operation: pending.operation as i32,
                replica_identity: pending.replica_identity as i32,
                before: pending.before,
                after: pending.after,
                idempotency_key: idempotency_key(
                    &config.source_id,
                    &commit_lsn,
                    &transaction.transaction_id,
                    total_order,
                ),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let ddl_events = transaction
        .ddl_events
        .into_iter()
        .map(|pending| {
            let mut event = pending.event;
            event.total_order = compacted_order(&order_map, pending.total_order)?;
            Ok(event)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: config.source_id.clone(),
        database_id: config.database_id.clone(),
        dataset_id: config.dataset_id.clone(),
        transaction_id: transaction.transaction_id,
        begin_lsn: transaction.begin_lsn,
        commit_lsn,
        commit_timestamp_ms,
        changes,
    });
    envelope.schema_versions = schema_versions;
    envelope.ddl_events = ddl_events;
    envelope.finalize_checksum();
    envelope.validate()?;
    Ok(envelope)
}
