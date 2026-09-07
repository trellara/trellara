use super::*;

#[test]
fn schema_discovery_suggests_contract_snippets() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = SchemaDiscoverySummary::from_preflight(
        &config,
        vec![
            preflight_table(42),
            preflight_table_named("public", "refunds"),
        ],
    );

    assert_eq!(summary.source_id, "local-source");
    assert_eq!(summary.dataset_id, "retail-sales");
    assert_eq!(summary.table_count, 2);
    assert_eq!(summary.tables[0].relation, "public.sales");
    assert!(summary.tables[0].configured);
    assert_eq!(summary.tables[0].replica_identity.as_deref(), Some("full"));
    assert_eq!(
        summary.tables[0].suggested_primary_key.as_deref(),
        Some("id")
    );
    assert_eq!(summary.tables[0].schema_fingerprint, Some(42));
    assert!(summary.tables[0]
        .config_snippet
        .contains("source_schema_fingerprint: 42"));
    assert!(summary.tables[0].config_snippet.contains("primary_key: id"));
    assert_eq!(summary.tables[1].relation, "public.refunds");
    assert!(!summary.tables[1].configured);
}

#[test]
fn schema_discovery_handles_missing_primary_key_suggestion() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
        primary_key_columns: Vec::new(),
        update_delete_safe: false,
        issues: vec!["primary key is missing".to_string()],
        ..preflight_table(77)
    };
    let summary = SchemaDiscoverySummary::from_preflight(&config, vec![table]);

    assert_eq!(summary.tables[0].suggested_primary_key, None);
    assert!(!summary.tables[0].update_delete_safe);
    assert!(summary.tables[0]
        .config_snippet
        .contains("source_schema_fingerprint: 77"));
    assert!(!summary.tables[0].config_snippet.contains("primary_key:"));
    assert_eq!(
        summary.tables[0].issues,
        vec!["primary key is missing".to_string()]
    );
}
