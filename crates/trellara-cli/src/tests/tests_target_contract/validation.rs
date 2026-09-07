use super::*;

#[test]
fn preflight_summary_failure_lists_contract_issues() {
    let table = trellara_pg_capture::TablePreflight {
        issues: vec!["source schema fingerprint mismatch: expected 99, got 42".to_string()],
        ..preflight_table(42)
    };

    assert!(matches!(
        fail_on_preflight_summary(&[table]),
        Err(CliError::InvalidConfig(message))
            if message == "preflight failed: public.sales: source schema fingerprint mismatch: expected 99, got 42"
    ));
}

#[test]
fn table_verify_primary_key_must_not_be_empty() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      verify:\n        primary_key: ' '",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.tables[].verify.primary_key must not be empty"
    ));
}

#[test]
fn table_verify_row_filter_must_not_be_empty() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      verify:\n        row_filter: ' '",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.tables[].verify.row_filter must not be empty"
    ));
}
