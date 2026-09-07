use super::*;

#[test]
fn table_selector_quotes_identifiers() {
    let selector = TableSelector::new("public", "sales");
    assert_eq!(selector.to_qualified_sql(), "\"public\".\"sales\"");

    let selector = TableSelector::new("odd\"schema", "odd\"table");
    assert_eq!(
        selector.to_qualified_sql(),
        "\"odd\"\"schema\".\"odd\"\"table\""
    );
}

#[test]
fn config_requires_tables() {
    let config = PgCaptureConfig {
        connection_uri: "postgres://example".to_string(),
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication_name: "pub".to_string(),
        slot_name: "slot".to_string(),
        tables: Vec::new(),
        create_if_missing: true,
        stream_spill_threshold_changes: DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    };

    assert!(matches!(
        config.validate(),
        Err(CaptureError::InvalidConfig(_))
    ));
}

#[test]
fn config_rejects_unbounded_stream_spill_threshold() {
    let config = PgCaptureConfig {
        connection_uri: "postgres://example".to_string(),
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication_name: "pub".to_string(),
        slot_name: "slot".to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: MAX_STREAM_SPILL_THRESHOLD_CHANGES + 1,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    };

    assert!(matches!(
        config.validate(),
        Err(CaptureError::InvalidConfig(message))
            if message.contains("stream_spill_threshold_changes")
                && message.contains("at most")
    ));
}
