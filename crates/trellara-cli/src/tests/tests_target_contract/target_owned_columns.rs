use super::*;

#[test]
fn target_owned_columns_are_not_required_from_source_but_must_exist_on_target() {
    let yaml = STRICT_YAML.replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      contract:\n        target_owned_columns: [reviewed_by]",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let clean = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            source_column("reviewed_by", "text"),
        ])],
        vec![target_inspection(vec![
            target_column("id", "text"),
            target_column("reviewed_by", "text"),
        ])],
    );
    let missing_target_owned = apply_target_contracts(
        &config,
        vec![preflight_table_with_columns(vec![
            source_column("id", "text"),
            source_column("reviewed_by", "text"),
        ])],
        vec![target_inspection(vec![target_column("id", "text")])],
    );

    assert!(clean[0].issues.is_empty());
    assert_eq!(
        missing_target_owned[0].issues,
        vec!["target-owned column reviewed_by does not exist on target".to_string()]
    );
}
