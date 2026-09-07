use std::path::PathBuf;

use super::*;
use trellara_protocol::{ChangeRecord, ColumnValue, ReplicaIdentity, RowImage, StrictEnvelope};

pub(super) struct LakeFaninRunFixture {
    pub(super) config_file: PathBuf,
    pub(super) envelope_file: PathBuf,
}

impl LakeFaninRunFixture {
    pub(super) fn new(prefix: &str, source_id: &str, transaction_id: &str) -> Self {
        let config_file = temp_file(prefix, "config", "yml");
        let envelope_file = temp_file(prefix, "envelope", "pb");
        fs::write(
            &config_file,
            STRICT_YAML.replace(
                "    - schema: public\n      name: sales",
                "    - schema: public\n      name: sales\n      verify:\n        primary_key: id",
            ),
        )
        .expect("write config");
        fs::write(
            &envelope_file,
            envelope(source_id, transaction_id)
                .encode_checked()
                .expect("encoded envelope"),
        )
        .expect("write envelope");
        Self {
            config_file,
            envelope_file,
        }
    }

    pub(super) fn remove(self) {
        fs::remove_file(self.config_file).expect("remove config");
        fs::remove_file(self.envelope_file).expect("remove envelope");
    }
}

fn envelope(source_id: &str, transaction_id: &str) -> TransactionEnvelope {
    let relation = trellara_protocol::RelationId::new(1, "public", "sales");
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail-sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: transaction_id.to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(relation),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: None,
            after: Some(RowImage::new(vec![
                ColumnValue::text("id", 23, "sale-1", true),
                ColumnValue::text("amount", 25, "10", false),
            ])),
            idempotency_key: format!("{source_id}:0/16B6C50:{transaction_id}:1"),
        }],
    })
}

fn temp_file(prefix: &str, kind: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{kind}-{}-{}.{}",
        std::process::id(),
        unique_test_suffix(),
        extension
    ))
}
