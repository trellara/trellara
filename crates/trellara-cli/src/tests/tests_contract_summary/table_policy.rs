use super::*;

#[test]
fn contract_test_rejects_unknown_table_by_default() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![
            preflight_table(42),
            preflight_table_named("public", "new_sales"),
        ],
    );

    assert!(!summary.passed);
    let unknown = summary
        .checks
        .iter()
        .find(|check| check.name == "source_table_policy:public.new_sales")
        .expect("unknown table check");
    assert_eq!(unknown.severity, ContractSeverity::Error);
    assert!(unknown.message.contains("not listed in dataset.tables"));
}

#[test]
fn contract_test_allows_unknown_table_when_policy_allows_compatible() {
    let yaml = STRICT_YAML.replace(
        "  mode: strict_transaction_order",
        "  mode: strict_transaction_order\n  unknown_table_policy: allow_compatible",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(
        &config,
        vec![preflight_table_named("public", "new_sales")],
    );

    assert!(summary.passed);
    let unknown = summary
        .checks
        .iter()
        .find(|check| check.name == "source_table_policy:public.new_sales")
        .expect("unknown table check");
    assert_eq!(unknown.severity, ContractSeverity::Warning);
    assert!(unknown.message.contains("allow_compatible permits it"));
}
