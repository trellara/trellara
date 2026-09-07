use super::*;

pub(super) fn ddl_envelope(
    transaction_id: &str,
    changes: Vec<ChangeRecord>,
    ddl_events: Vec<DdlEvent>,
) -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    });
    envelope.ddl_events = ddl_events;
    envelope.finalize_checksum();
    envelope
}

pub(super) fn additive_ddl(transaction_id: &str, total_order: u32) -> DdlEvent {
    DdlEvent::additive_column(
        transaction_id,
        total_order,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )
}
