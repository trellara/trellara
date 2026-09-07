use super::*;

#[test]
fn lake_ddl_summary_generates_raw_cdc_and_epoch_metadata_specs() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakeDdlSummary::from_config(&config);

    assert_eq!(summary.source_id, "local-source");
    assert_eq!(summary.dataset_id, "retail-sales");
    assert_eq!(summary.mode, "strict_transaction_order");
    assert_eq!(
        summary.contract,
        "fleet_fanin_append_only_raw_cdc_with_epoch_completeness"
    );
    assert_eq!(summary.table_count, 7);
    assert!(summary
        .tables
        .iter()
        .any(|table| table.materialization == "raw_cdc_append_only"));
    assert!(summary
        .tables
        .iter()
        .any(|table| table.materialization == "epoch_metadata"));
    assert!(summary.tables.iter().all(|table| table
        .visibility_boundary
        .contains("source transaction envelope")));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__public__sales__raw_cdc"
            && table.primary_key.as_deref() == Some("id")
            && table.partitioning
                == vec![
                    "bucket(32, __trellara_source_id)".to_string(),
                    "days(__trellara_commit_timestamp)".to_string(),
                    "__trellara_relation".to_string(),
                ]
            && table.ddl.contains("__trellara_idempotency_key")
            && table.ddl.contains("__trellara_epoch_id")
            && table.ddl.contains("__trellara_envelope_checksum")
            && table.ddl.contains("__trellara_schema_version")
            && table.ddl.contains("__trellara_ddl_barrier_id")
            && table.ddl.contains("__trellara_ddl_release_gate")
            && table
                .ddl
                .contains("__trellara_ddl_schema_fingerprint_before")
            && table
                .ddl
                .contains("__trellara_ddl_schema_fingerprint_after")
            && table.ddl.contains("__trellara_payload_before")
            && table.ddl.contains("__trellara_payload_after")
            && table.ddl.contains("__trellara_manifest_boundary_mode")
            && table.ddl.contains("__trellara_manifest_global_event_count")
            && table
                .ddl
                .contains("__trellara_manifest_participating_partition_count")
            && !table.ddl.contains("row_before_json")
            && !table.ddl.contains("row_after_json")
    }));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__trellara__fanin___trellara_epoch_sources"
            && table.materialization == "epoch_source_metadata"
            && table.ddl.contains("start_lsn TEXT NOT NULL")
            && table.ddl.contains("end_lsn TEXT NOT NULL")
            && table.ddl.contains("lag_reason TEXT")
    }));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__trellara__fanin___trellara_epochs"
            && table.materialization == "epoch_metadata"
            && table.ddl.contains("manifest_digest TEXT NOT NULL")
            && table.ddl.contains("iceberg_snapshot_id TEXT NOT NULL")
            && table
                .ddl
                .contains("raw_table_snapshot_ids_json TEXT NOT NULL")
            && table.partitioning == vec!["unpartitioned_l1_metadata_release_marker".to_string()]
    }));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__trellara__fanin___trellara_epoch_partitions"
            && table.materialization == "epoch_partition_metadata"
            && table.ddl.contains("source_id TEXT NOT NULL")
            && table.ddl.contains("partition_id BIGINT NOT NULL")
            && table.ddl.contains("first_commit_lsn TEXT NOT NULL")
            && table.ddl.contains("last_commit_lsn TEXT NOT NULL")
            && table.ddl.contains("event_count BIGINT NOT NULL")
            && table.partitioning
                == vec![
                    "epoch_id".to_string(),
                    "bucket(64, source_id)".to_string(),
                    "partition_id".to_string(),
                ]
    }));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__trellara__fanin___trellara_quarantine"
            && table.materialization == "epoch_quarantine_metadata"
            && table.ddl.contains("source_id TEXT")
            && table.ddl.contains("transaction_id TEXT")
            && table.ddl.contains("commit_lsn TEXT")
            && table.ddl.contains("reason TEXT NOT NULL")
            && table.ddl.contains("details TEXT")
            && table.ddl.contains("recovery_command TEXT")
            && table.partitioning == vec!["epoch_id".to_string(), "reason".to_string()]
    }));
    assert_eq!(
        summary.checkpoint_contract.checkpoint_column,
        "__trellara_commit_lsn"
    );
    assert!(summary
        .checkpoint_contract
        .idempotency_rule
        .contains("__trellara_idempotency_key"));
    assert_eq!(
        summary.spark_template_outputs,
        vec![
            "retail_sales__spark__derived__current_state",
            "retail_sales__spark__derived__scd2_history"
        ]
    );
}

#[test]
fn lake_ddl_summary_preserves_partitioned_visibility_contract() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = LakeDdlSummary::from_config(&config);

    assert_eq!(summary.mode, "partitioned_scale_mode");
    assert!(summary
        .checkpoint_contract
        .visibility_rule
        .contains("global low watermark"));
    assert!(summary
        .tables
        .iter()
        .all(|table| table.visibility_boundary.contains("global low watermark")));
    assert!(summary.tables.iter().any(|table| {
        table.table_name == "retail_sales__public__sales__raw_cdc"
            && table
                .partitioning
                .contains(&"bucket(32, __trellara_partition_key)".to_string())
            && table.ddl.contains("__trellara_manifest_boundary_mode TEXT")
            && !table
                .partitioning
                .contains(&"bucket(32, __trellara_source_id)".to_string())
    }));
}
