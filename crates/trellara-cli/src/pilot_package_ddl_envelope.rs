use trellara_protocol::{DdlEvent, RelationId, StrictEnvelope, TransactionEnvelope};

use crate::{DdlEnvelopePlanSummary, Result, TrellaraConfig};

pub(crate) fn pilot_package_schema_ddl_envelope_plan(
    config: &TrellaraConfig,
) -> Result<DdlEnvelopePlanSummary> {
    DdlEnvelopePlanSummary::from_envelope(config, &pilot_package_schema_ddl_envelope(config))
}

fn pilot_package_schema_ddl_envelope(config: &TrellaraConfig) -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: config.source.id.clone(),
        database_id: config
            .source
            .database_id
            .clone()
            .unwrap_or_else(|| "postgres".to_string()),
        dataset_id: config.dataset.id.clone(),
        transaction_id: "tx-schema-ddl-sample".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-schema-ddl-sample",
        1,
        RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}
