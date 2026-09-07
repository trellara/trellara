use super::*;
use trellara_protocol::{
    AffectedTable, ChangeRecord, ColumnValue, ManifestBoundaryMode, ManifestPartition,
    ReplicaIdentity, RowImage, StrictEnvelope, TransactionManifest,
};

#[tokio::test]
async fn lake_inspect_command_preserves_strict_chunk_visibility_boundary() {
    let config_file = std::env::temp_dir().join(format!(
        "trellara-lake-strict-chunk-config-{}-{}.yml",
        std::process::id(),
        "tx-lake-chunk"
    ));
    let envelope_file = std::env::temp_dir().join(format!(
        "trellara-lake-strict-chunk-envelope-{}-{}.pb",
        std::process::id(),
        "tx-lake-chunk"
    ));
    fs::write(
        &config_file,
        local_strict_chunking_yaml().replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      verify:\n        primary_key: id",
        ),
    )
    .expect("write config");

    let relation = trellara_protocol::RelationId::new(1, "public", "sales");
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "local-source".to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail-sales".to_string(),
        transaction_id: "tx-lake-chunk".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![
            ChangeRecord {
                transaction_id: "tx-lake-chunk".to_string(),
                total_order: 1,
                table_order: 1,
                partition_order: 1,
                relation: Some(relation.clone()),
                operation: Operation::Insert as i32,
                replica_identity: ReplicaIdentity::Full as i32,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-1", true,
                )])),
                idempotency_key: "local-source:0/16B6C50:tx-lake-chunk:1".to_string(),
            },
            ChangeRecord {
                transaction_id: "tx-lake-chunk".to_string(),
                total_order: 2,
                table_order: 2,
                partition_order: 2,
                relation: Some(relation.clone()),
                operation: Operation::Insert as i32,
                replica_identity: ReplicaIdentity::Full as i32,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-2", true,
                )])),
                idempotency_key: "local-source:0/16B6C50:tx-lake-chunk:2".to_string(),
            },
        ],
    });
    envelope.manifest = Some(TransactionManifest {
        transaction_id: "tx-lake-chunk".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_786_420_000_000,
        global_event_count: 2,
        partitions: vec![
            ManifestPartition {
                id: 0,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 99,
            },
            ManifestPartition {
                id: 1,
                event_count: 1,
                first_total_order: 2,
                last_total_order: 2,
                checksum: 100,
            },
        ],
        affected_tables: vec![AffectedTable {
            relation: Some(relation),
            event_count: 2,
        }],
        boundary_mode: ManifestBoundaryMode::StrictChunkedTransactionOrder as i32,
    });
    envelope.finalize_checksum();
    fs::write(
        &envelope_file,
        envelope.encode_checked().expect("encoded envelope"),
    )
    .expect("write envelope");
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "inspect",
        "--config",
        config_file.to_str().expect("utf8 config path"),
        "--file",
        envelope_file.to_str().expect("utf8 envelope path"),
    ])
    .expect("cli parse");

    let output = execute(cli).await.expect("execute");

    assert!(output.contains("\"kind\": \"strict_chunk_manifest_barrier\""));
    assert!(output.contains("\"global_event_count\": 2"));
    assert!(output.contains("\"chunk_count\": 2"));
    assert!(!output.contains("participating_partition_count"));
    fs::remove_file(config_file).expect("remove config");
    fs::remove_file(envelope_file).expect("remove envelope");
}
