use super::*;
use std::fs;
use std::path::PathBuf;
use trellara_protocol::{ColumnValue, Operation, ReplicaIdentity, RowImage};

pub(super) fn assembler_config() -> TransactionAssemblerConfig {
    TransactionAssemblerConfig {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
    }
}

pub(super) fn stream_spill_test_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("trellara-pg-capture-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create spill test dir");
    root
}

pub(super) fn record_sales_relation_metadata(
    assembler: &mut TransactionAssembler,
    config: &TransactionAssemblerConfig,
) {
    assembler
        .apply(
            config,
            LogicalEvent::RelationMetadata {
                relation: RelationId::new(16_384, "public", "sales"),
                schema_fingerprint: 12_345,
            },
        )
        .expect("sales relation metadata");
}

pub(super) fn streamed_insert_event(transaction_id: &str, value: &str) -> LogicalEvent {
    LogicalEvent::Change {
        transaction_id: Some(transaction_id.to_string()),
        relation: RelationId::new(16_384, "public", "sales"),
        operation: Operation::Insert,
        replica_identity: ReplicaIdentity::Default,
        before: None,
        after: Some(RowImage::new(vec![ColumnValue::text(
            "id", 23, value, true,
        )])),
    }
}
