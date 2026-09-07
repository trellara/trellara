use super::*;

#[test]
fn schema_contract_mismatch_adds_preflight_issue() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        source_schema_fingerprint: 99",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let tables = apply_schema_contracts(&config, vec![preflight_table(42)]);

    assert_eq!(tables[0].issues.len(), 1);
    assert_eq!(
        tables[0].issues[0],
        "source schema fingerprint mismatch: expected 99, got 42"
    );
}

#[test]
fn schema_contract_match_keeps_preflight_clean() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        source_schema_fingerprint: 42",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let tables = apply_schema_contracts(&config, vec![preflight_table(42)]);

    assert!(tables[0].issues.is_empty());
}

#[test]
fn contract_test_recommends_pinning_missing_source_schema_fingerprints() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert_eq!(summary.recovery_action_count, 1);
    let action = summary
        .recovery_actions
        .iter()
        .find(|action| action.code == "source_schema_fingerprint_pin")
        .expect("fingerprint pin action");
    assert_eq!(action.severity, ContractSeverity::Warning);
    assert_eq!(action.relations, vec!["public.sales".to_string()]);
    assert_eq!(
        action.command_templates,
        vec![
            "trellara schema-discover --config <config>".to_string(),
            "trellara contract-test --config <config>".to_string(),
        ]
    );
    assert!(action.hint.contains("source_schema_fingerprint"));
}

#[test]
fn contract_test_scripts_schema_handoff_when_pinned_fingerprint_drifts() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        source_schema_fingerprint: 99",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let tables = apply_schema_contracts(&config, vec![preflight_table(42)]);
    let summary = ContractTestSummary::from_preflight(&config, tables);

    assert!(!summary.passed);
    let action = summary
        .recovery_actions
        .iter()
        .find(|action| action.code == "source_schema_handoff_required")
        .expect("schema handoff action");
    assert_eq!(action.severity, ContractSeverity::Error);
    assert_eq!(action.relations, vec!["public.sales".to_string()]);
    assert_eq!(
        action.command_templates,
        vec![
            "trellara schema-discover --config <config>".to_string(),
            "trellara contract-test --config <config>".to_string(),
            "trellara snapshot --config <config> --force".to_string(),
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    );
    assert!(action
        .hint
        .contains("fresh audited snapshot-to-stream handoff"));
}

#[test]
fn source_schema_drift_summary_sorts_and_deduplicates_relations() {
    let mut sales = preflight_table_named("public", "sales");
    sales
        .issues
        .push("source schema fingerprint mismatch: expected 99, got 42".to_string());
    let mut orders = preflight_table_named("public", "orders");
    orders
        .issues
        .push("source schema fingerprint mismatch: expected 99, got 42".to_string());

    let drift = FlowSchemaDriftSummary::from_preflight(&[sales.clone(), orders, sales])
        .expect("schema drift");

    assert_eq!(drift.relation_count, 2);
    assert_eq!(
        drift.relations,
        vec!["public.orders".to_string(), "public.sales".to_string()]
    );
}
