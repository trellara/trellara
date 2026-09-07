use super::*;

#[test]
fn selected_configured_tables_returns_all_tables_without_filter() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    let selected = selected_configured_tables(&config.dataset.tables, None, "verify --table")
        .expect("select tables");

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].relation_id().display_name(), "public.sales");
}

#[test]
fn selected_configured_tables_filters_by_relation_name() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    let selected = selected_configured_tables(
        &config.dataset.tables,
        Some("public.sales"),
        "verify --table",
    )
    .expect("select tables");

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, "sales");
}

#[test]
fn selected_configured_tables_rejects_unknown_relation() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    let error = selected_configured_tables(
        &config.dataset.tables,
        Some("public.unknown"),
        "verify --table",
    )
    .expect_err("unknown table rejected");

    assert!(matches!(
        error,
        CliError::InvalidConfig(message)
            if message.contains("public.unknown")
                && message.contains("verify --table")
                && message.contains("dataset.tables")
                && message.contains("public.sales")
    ));
}
