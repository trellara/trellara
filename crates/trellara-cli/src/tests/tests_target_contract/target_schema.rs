use super::*;

#[test]
fn missing_target_table_adds_preflight_issue() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let tables = apply_target_contracts(
        &config,
        vec![preflight_table(42)],
        vec![missing_target_inspection()],
    );

    assert_eq!(
        tables[0].issues,
        vec!["target table does not exist".to_string()]
    );
}

#[test]
fn missing_target_column_adds_preflight_issue() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let tables = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            source_column("amount_cents", "int8"),
        ])],
        vec![target_inspection(vec![target_column("id", "text")])],
    );

    assert_eq!(
        tables[0].issues,
        vec!["target table is missing required column amount_cents".to_string()]
    );
}

#[test]
fn target_type_mismatch_adds_preflight_issue() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let tables = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            source_column("amount_cents", "int8"),
        ])],
        vec![target_inspection(vec![
            target_column("id", "text"),
            target_column("amount_cents", "bool"),
        ])],
    );

    assert_eq!(
        tables[0].issues,
        vec!["target column amount_cents type mismatch: source int8, target bool".to_string()]
    );
}

#[test]
fn compatible_target_schema_drift_keeps_preflight_clean() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut reviewed_by = target_column("reviewed_by", "text");
    reviewed_by.nullable = true;
    let tables = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            source_column("amount_cents", "int4"),
            source_column("sku", "varchar(64)"),
        ])],
        vec![target_inspection(vec![
            target_column("id", "text"),
            target_column("amount_cents", "int8"),
            target_column("sku", "text"),
            reviewed_by,
        ])],
    );

    assert!(tables[0].issues.is_empty());
    assert_eq!(
        tables[0].contract_notes,
        vec![
            "compatible target type widening for amount_cents: source int4, target int8"
                .to_string(),
            "compatible target type widening for sku: source varchar(64), target text".to_string(),
            "compatible nullable target-only column reviewed_by".to_string(),
        ]
    );
}

#[test]
fn contract_test_surfaces_compatible_schema_drift_notes() {
    let table = trellara_pg_capture::TablePreflight {
        contract_notes: vec![
            "compatible target type widening for amount_cents: source int4, target int8"
                .to_string(),
        ],
        ..preflight_table(42)
    };
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(summary.passed);
    let drift = summary
        .checks
        .iter()
        .find(|check| check.name == "schema_compatibility:public.sales")
        .expect("schema compatibility check");
    assert!(drift.passed);
    assert_eq!(drift.severity, ContractSeverity::Warning);
    assert!(drift.message.contains("compatible target type widening"));
}

#[test]
fn stricter_target_only_column_adds_preflight_issue() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let tables = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![source_column(
            "id", "text",
        )])],
        vec![target_inspection(vec![
            target_column("id", "text"),
            target_column("approval_code", "text"),
        ])],
    );

    assert_eq!(
        tables[0].issues,
        vec!["target-only column approval_code is non-null and not target-owned".to_string()]
    );
}
