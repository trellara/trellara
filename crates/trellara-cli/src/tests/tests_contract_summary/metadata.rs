use super::*;

#[test]
fn contract_test_names_live_pgoutput_relation_metadata_fingerprint() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert!(summary.passed);
    let metadata = summary
        .checks
        .iter()
        .find(|check| check.name == "pgoutput_relation_metadata:public.sales")
        .expect("pgoutput relation metadata check");
    assert!(metadata.passed);
    assert_eq!(metadata.severity, ContractSeverity::Info);
    assert!(metadata
        .message
        .contains("live pgoutput relation metadata fingerprint 42"));
}

#[test]
fn contract_test_warns_when_pgoutput_relation_metadata_fingerprint_is_missing() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
        schema_fingerprint: None,
        ..preflight_table(42)
    };
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(summary.passed);
    let metadata = summary
        .checks
        .iter()
        .find(|check| check.name == "pgoutput_relation_metadata:public.sales")
        .expect("pgoutput relation metadata check");
    assert!(metadata.passed);
    assert_eq!(metadata.severity, ContractSeverity::Warning);
    assert!(metadata
        .message
        .contains("pinning and drift checks cannot be proven"));
}

#[test]
fn contract_test_surfaces_row_filter_as_contract_info() {
    let yaml = STRICT_YAML.replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      verify:\n        row_filter: \"store_id = 'store-1'\"",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert!(summary.passed);
    assert_eq!(summary.issue_count, 0);
    let row_filter = summary
        .checks
        .iter()
        .find(|check| check.name == "row_filter:public.sales")
        .expect("row filter check");
    assert_eq!(row_filter.severity, ContractSeverity::Info);
    assert!(row_filter.message.contains("store_id = 'store-1'"));
}
