use super::*;
use trellara_protocol::{ChangeRecord, ColumnValue, ReplicaIdentity, RowImage, StrictEnvelope};

#[tokio::test]
async fn lake_inspect_command_plans_materializations_from_encoded_envelope() {
    let config_file = std::env::temp_dir().join(format!(
        "trellara-lake-config-{}-{}.yml",
        std::process::id(),
        "tx-lake"
    ));
    let envelope_file = std::env::temp_dir().join(format!(
        "trellara-lake-envelope-{}-{}.pb",
        std::process::id(),
        "tx-lake"
    ));
    fs::write(
        &config_file,
        STRICT_YAML.replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      verify:\n        primary_key: id\n        excluded_columns: [updated_at]",
        ),
    )
    .expect("write config");

    let relation = trellara_protocol::RelationId::new(1, "public", "sales");
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "local-source".to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail-sales".to_string(),
        transaction_id: "tx-lake".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-lake".to_string(),
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
                ColumnValue::text("updated_at", 25, "2026-01-01", false),
            ])),
            idempotency_key: "local-source:0/16B6C50:tx-lake:1".to_string(),
        }],
    });
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

    assert!(output.contains("\"transaction_id\": \"tx-lake\""));
    assert!(output.contains("\"operation_count\": 3"));
    assert!(output.contains("\"kind\": \"strict_envelope\""));
    assert!(output.contains("\"materialization\": \"raw_cdc\""));
    assert!(output.contains("\"materialization\": \"current_state\""));
    assert!(output.contains("\"materialization\": \"scd2_history\""));
    assert!(output.contains("\"record_key\": \"sale-1\""));
    assert!(!output.contains("updated_at"));
    fs::remove_file(config_file).expect("remove config");
    fs::remove_file(envelope_file).expect("remove envelope");
}
