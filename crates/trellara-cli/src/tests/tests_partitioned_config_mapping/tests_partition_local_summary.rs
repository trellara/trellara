use super::*;

#[test]
fn partition_local_summary_is_disabled_for_strict_mode() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = PartitionLocalSummary::from_config(&config).expect("summary");

    assert!(!summary.enabled);
    assert_eq!(summary.mode, "strict_transaction_order");
    assert_eq!(summary.key_column, None);
    assert_eq!(summary.partition_count, None);
    assert_eq!(summary.null_key_policy, None);
    assert_eq!(summary.key_change_policy, None);
    assert_eq!(summary.manifest_topic, None);
    assert!(summary.limitations[0].contains("multi-partition"));
}

#[test]
fn partition_local_summary_reports_topics_and_semantics() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = PartitionLocalSummary::from_config(&config).expect("summary");

    assert!(summary.enabled);
    assert_eq!(summary.mode, "partitioned_scale_mode");
    assert_eq!(summary.key_column, Some("store_id".to_string()));
    assert_eq!(summary.partition_count, Some(4));
    assert_eq!(summary.null_key_policy, Some("quarantine".to_string()));
    assert_eq!(summary.key_change_policy, Some("quarantine".to_string()));
    assert_eq!(
        summary.manifest_topic,
        Some("trellara.local-source.retail-sales.manifest".to_string())
    );
    assert_eq!(
        summary.partition_topic_pattern,
        Some("trellara.local-source.retail-sales.partition.<0..3>".to_string())
    );
    assert!(summary.guarantees[0].contains("only their own lane"));
}

#[test]
fn partition_local_summary_reports_configured_partition_policies() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: derive_from_primary_key\n    key_change_policy: emit_move",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = PartitionLocalSummary::from_config(&config).expect("summary");

    assert_eq!(
        summary.null_key_policy,
        Some("derive_from_primary_key".to_string())
    );
    assert_eq!(summary.key_change_policy, Some("emit_move".to_string()));
}
