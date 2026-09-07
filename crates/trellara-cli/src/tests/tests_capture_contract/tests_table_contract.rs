use super::*;

#[test]
fn parses_table_verify_settings() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      verify:\n        primary_key: sale_id\n        excluded_columns: [updated_at, ingested_at]\n        row_filter: \"region = 'west'\"",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let verify = config.dataset.tables[0]
        .verify
        .as_ref()
        .expect("verify config");

    assert_eq!(verify.primary_key, "sale_id");
    assert_eq!(verify.excluded_columns, vec!["updated_at", "ingested_at"]);
    assert_eq!(verify.row_filter.as_deref(), Some("region = 'west'"));
}

#[test]
fn parses_table_schema_contract() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        source_schema_fingerprint: 42\n        target_owned_columns: [reviewed_by, reviewed_at]",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let contract = config.dataset.tables[0]
        .contract
        .as_ref()
        .expect("contract");

    assert_eq!(contract.source_schema_fingerprint, Some(42));
    assert_eq!(
        contract.target_owned_columns,
        vec!["reviewed_by", "reviewed_at"]
    );
    config.validate().expect("valid contract");
}

#[test]
fn table_schema_contract_fingerprint_must_be_nonzero() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        source_schema_fingerprint: 0",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.tables[].contract.source_schema_fingerprint must be greater than zero"
    ));
}

#[test]
fn table_schema_contract_target_owned_columns_must_not_be_empty() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        target_owned_columns: [' ']",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.tables[].contract.target_owned_columns[] must not be empty"
    ));
}

#[test]
fn target_owned_columns_map_to_apply_policy() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      contract:\n        target_owned_columns: [reviewed_by]",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let apply = config
        .to_postgres_apply_config("postgres://target".to_string(), true)
        .expect("apply config");

    assert_eq!(apply.table_policies.len(), 1);
    assert_eq!(
        apply.table_policies[0].relation.display_name(),
        "public.sales"
    );
    assert_eq!(
        apply.table_policies[0].target_owned_columns,
        vec!["reviewed_by".to_string()]
    );
}
