use super::*;

#[test]
fn contract_test_validates_partition_key_contract() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            key_column("store_id", false),
        ])],
    );

    assert!(summary.passed);
    assert!(summary
        .checks
        .iter()
        .any(|check| check.name == "transaction_boundary:partition_manifest"));
    assert!(summary.checks.iter().any(|check| {
        check.name == "partition_policy:null_key"
            && check
                .message
                .contains("null partition keys use policy quarantine")
    }));
    assert!(summary.checks.iter().any(|check| {
        check.name == "partition_policy:key_change"
            && check
                .message
                .contains("partition key changes use policy quarantine")
    }));
    assert!(summary
        .checks
        .iter()
        .any(|check| check.name == "partition_key:public.sales"));
}

#[test]
fn contract_test_rejects_primary_key_derivation_without_primary_key() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: derive_from_primary_key",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![preflight_table_with_columns({
            let mut id = source_column("id", "text");
            id.is_key = false;
            let mut store_id = source_column("store_id", "text");
            store_id.nullable = true;
            vec![id, store_id]
        })],
    );

    assert!(!summary.passed);
    let partition_key = summary
        .checks
        .iter()
        .find(|check| check.name == "partition_key:public.sales")
        .expect("partition key check");
    assert_eq!(partition_key.severity, ContractSeverity::Error);
    assert!(partition_key
        .message
        .contains("no primary key is available"));
}

#[test]
fn contract_test_flags_missing_partition_key() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert!(!summary.passed);
    assert_eq!(summary.issue_count, 1);
    assert_eq!(
        summary.checks.last().expect("partition check").severity,
        ContractSeverity::Error
    );
    assert!(summary
        .checks
        .last()
        .expect("partition check")
        .message
        .contains("missing partition key column store_id"));
}
