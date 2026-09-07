use super::*;

mod identity;
mod metadata;
mod table_policy;
mod transaction_boundary;

#[test]
fn contract_test_passes_for_clean_strict_flow() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert!(summary.passed);
    assert_eq!(summary.issue_count, 0);
    assert!(summary
        .checks
        .iter()
        .any(|check| check.name == "transaction_boundary:strict"));
}
