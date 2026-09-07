use super::*;

#[test]
fn init_renders_valid_local_config() {
    let args = init_args(PathBuf::from("trellara.yml"));
    let yaml = render_init_config(&args).expect("render init config");
    let config = TrellaraConfig::from_yaml(&yaml, "init").expect("parse generated config");

    config.validate().expect("valid generated config");
    assert_eq!(config.source.id, "store-fleet");
    assert_eq!(config.source.database_id.as_deref(), Some("retail"));
    assert_eq!(
        config.source.stream_spill_threshold_changes,
        Some(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES)
    );
    assert_eq!(
        config.source.stream_spill_dir.as_deref(),
        Some(Path::new("./target/trellara-spill"))
    );
    assert_eq!(config.dataset.id, "sales");
    assert_eq!(
        config
            .dataset
            .strict_chunking
            .as_ref()
            .expect("strict chunking")
            .max_changes_per_chunk,
        1000
    );
    assert_eq!(config.dataset.tables.len(), 2);
    assert_eq!(
        config.dataset.tables[0]
            .verify
            .as_ref()
            .expect("verify")
            .primary_key,
        "id"
    );
    assert!(matches!(config.stream, StreamConfig::Local { .. }));
    assert_eq!(
        config.target.as_ref().expect("target").database_url,
        "postgresql://target/app"
    );
}

#[test]
fn init_rejects_malformed_table_names() {
    assert!(matches!(
        parse_init_table("public.sales.extra"),
        Err(CliError::InvalidConfig(message))
            if message.contains("schema.table")
    ));
    assert!(matches!(
        parse_init_table("sales"),
        Err(CliError::InvalidConfig(message))
            if message.contains("schema.table")
    ));
}
