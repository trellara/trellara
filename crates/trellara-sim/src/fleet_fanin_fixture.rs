use trellara_protocol::{
    idempotency_key, AffectedTable, ChangeRecord, ColumnValue, ManifestBoundaryMode,
    ManifestPartition, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope, TransactionManifest,
};

pub(crate) fn fleet_envelope(
    seed: u64,
    dataset_id: &str,
    source_id: &str,
    source_ordinal: usize,
    conflicting_payload: bool,
) -> TransactionEnvelope {
    let commit_lsn = format!("0/{:X}", 0x16b6c50 + (source_ordinal as u64 * 16));
    let transaction_id = format!("fleet-tx-{source_ordinal:04}");
    let amount = if conflicting_payload {
        format!("{}", 90_000 + source_ordinal)
    } else {
        format!("{}", 10_000 + source_ordinal)
    };
    let change = ChangeRecord {
        transaction_id: transaction_id.clone(),
        total_order: 1,
        table_order: 1,
        partition_order: 1,
        relation: Some(RelationId::new(10_000, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Full as i32,
        before: None,
        after: Some(RowImage::new(vec![
            ColumnValue::text("id", 25, format!("{source_id}-sale-{source_ordinal}"), true),
            ColumnValue::text("store_id", 25, source_id, false),
            ColumnValue::text("amount_cents", 20, amount, false),
        ])),
        idempotency_key: idempotency_key(source_id, &commit_lsn, &transaction_id, 1),
    };
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.to_string(),
        database_id: "postgres".to_string(),
        dataset_id: dataset_id.to_string(),
        transaction_id: transaction_id.clone(),
        begin_lsn: format!("0/{:X}", 0x16b6b00 + seed % 31),
        commit_lsn: commit_lsn.clone(),
        commit_timestamp_ms: 1_786_857_600_000 + source_ordinal as i64,
        changes: vec![change],
    });
    envelope.manifest = Some(TransactionManifest {
        transaction_id,
        source_commit_lsn: commit_lsn,
        source_commit_timestamp_ms: envelope.commit_timestamp_ms,
        global_event_count: 1,
        partitions: vec![ManifestPartition {
            id: ((source_ordinal - 1) % 8) as u32,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 1,
            checksum: envelope.changes[0].total_order as u64 ^ envelope.checksum,
        }],
        affected_tables: vec![AffectedTable {
            relation: Some(RelationId::new(10_000, "public", "sales")),
            event_count: 1,
        }],
        boundary_mode: ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();
    envelope
}

pub(crate) fn fleet_conflicting_duplicate(original: &TransactionEnvelope) -> TransactionEnvelope {
    let source_ordinal = original
        .transaction_id
        .strip_prefix("fleet-tx-")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let mut conflicting = fleet_envelope(
        0,
        &original.dataset_id,
        &original.source_id,
        source_ordinal,
        true,
    );
    conflicting.begin_lsn = original.begin_lsn.clone();
    conflicting.commit_lsn = original.commit_lsn.clone();
    conflicting.transaction_id = original.transaction_id.clone();
    conflicting.changes[0].transaction_id = original.transaction_id.clone();
    conflicting.changes[0].idempotency_key = original.changes[0].idempotency_key.clone();
    conflicting.finalize_checksum();
    conflicting
}
