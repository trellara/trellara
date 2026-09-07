use trellara_protocol::{
    ChangeRecord, ColumnValue, DdlEvent, Operation, RelationId, RelationSchemaVersion,
    ReplicaIdentity, RowImage, StrictEnvelope, TransactionEnvelope,
};

use crate::{CliError, Result, TrellaraConfig};

pub(crate) fn pilot_package_lake_writer_sample_envelope(
    config: &TrellaraConfig,
) -> Result<TransactionEnvelope> {
    let table = config.dataset.tables.first().ok_or_else(|| {
        CliError::InvalidConfig("dataset.tables must include at least one table".to_string())
    })?;
    let primary_key = table
        .verify
        .as_ref()
        .map(|verify| verify.primary_key.as_str())
        .filter(|primary_key| !primary_key.trim().is_empty())
        .unwrap_or("id");
    let source_id = config.source.id.clone();
    let transaction_id = "tx-lake-writer-sample".to_string();
    let commit_lsn = "0/16B6C50".to_string();

    let relation = RelationId::new(1, &table.schema, &table.name);
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.clone(),
        database_id: config
            .source
            .database_id
            .clone()
            .unwrap_or_else(|| "postgres".to_string()),
        dataset_id: config.dataset.id.clone(),
        transaction_id: transaction_id.clone(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.clone(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: transaction_id.clone(),
            total_order: 2,
            table_order: 1,
            partition_order: 2,
            relation: Some(relation.clone()),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: None,
            after: Some(RowImage::new(vec![
                ColumnValue::text(primary_key, 25, "sample-1", true),
                ColumnValue::text("trellara_sample_amount", 25, "10", false),
            ])),
            idempotency_key: format!("{source_id}:{commit_lsn}:{transaction_id}:2"),
        }],
    });
    envelope.schema_versions = vec![RelationSchemaVersion {
        relation: Some(relation.clone()),
        version: 67_890,
    }];
    envelope.ddl_events = vec![DdlEvent::additive_column(
        &transaction_id,
        1,
        relation,
        format!(
            "alter table {}.{} add column trellara_sample_note text",
            table.schema, table.name
        ),
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    Ok(envelope)
}
