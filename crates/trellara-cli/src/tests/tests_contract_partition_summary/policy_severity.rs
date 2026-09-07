use super::*;

#[test]
fn contract_test_reports_partition_policy_severity() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: derive_from_primary_key\n    key_change_policy: dual_write_window",
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
    let key_change = summary
        .checks
        .iter()
        .find(|check| check.name == "partition_policy:key_change")
        .expect("key change policy check");
    assert_eq!(key_change.severity, ContractSeverity::Warning);
    assert!(key_change.message.contains("dual_write_window"));
    assert!(summary.checks.iter().any(|check| {
        check.name == "partition_policy:null_key"
            && check.message.contains("derive_from_primary_key")
    }));
}

#[test]
fn contract_test_allows_nullable_partition_key_with_singleton_policy() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: route_to_singleton_partition",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            key_column("store_id", true),
        ])],
    );

    assert!(summary.passed);
    let partition_key = summary
        .checks
        .iter()
        .find(|check| check.name == "partition_key:public.sales")
        .expect("partition key check");
    assert!(partition_key.passed);
    assert_eq!(partition_key.severity, ContractSeverity::Warning);
    assert!(partition_key.message.contains("singleton partition 0"));
}

#[test]
fn contract_test_allows_nullable_partition_key_with_dead_letter_policy() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: route_to_dead_letter_partition",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            key_column("store_id", true),
        ])],
    );

    assert!(summary.passed);
    let partition_key = summary
        .checks
        .iter()
        .find(|check| check.name == "partition_key:public.sales")
        .expect("partition key check");
    assert_eq!(partition_key.severity, ContractSeverity::Warning);
    assert!(partition_key.message.contains("dead-letter partition"));
}

#[test]
fn contract_test_allows_nullable_partition_key_derived_from_primary_key() {
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
            let mut store_id = source_column("store_id", "text");
            store_id.nullable = true;
            vec![key_column("id", false), store_id]
        })],
    );

    assert!(summary.passed);
    let partition_key = summary
        .checks
        .iter()
        .find(|check| check.name == "partition_key:public.sales")
        .expect("partition key check");
    assert_eq!(partition_key.severity, ContractSeverity::Warning);
    assert!(partition_key.message.contains("primary-key columns"));
}
