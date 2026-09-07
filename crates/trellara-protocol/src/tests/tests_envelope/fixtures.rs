use super::*;

pub(super) fn envelope_with(
    transaction_id: &str,
    begin_lsn: &str,
    commit_lsn: &str,
    changes: Vec<ChangeRecord>,
) -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: begin_lsn.to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    })
}

pub(super) fn default_envelope() -> TransactionEnvelope {
    envelope_with("tx-1", "0/16B6B00", "0/16B6C50", vec![sample_change(1)])
}

pub(super) fn schema_version(version: u64) -> RelationSchemaVersion {
    RelationSchemaVersion {
        relation: Some(RelationId::new(16_384, "public", "sales")),
        version,
    }
}
