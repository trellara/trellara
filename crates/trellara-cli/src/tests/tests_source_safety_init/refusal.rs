use super::*;

#[test]
fn source_safety_write_init_refuses_overwrite_without_force() {
    let root = std::env::temp_dir().join(format!(
        "trellara-source-safety-overwrite-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let output = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create write-init temp dir");
    fs::write(&output, "existing").expect("write existing config");
    let args = SourceSafetyArgs {
        config: None,
        format: SourceSafetyOutputFormat::Json,
        output: None,
        database_url: Some("postgresql://source/app".to_string()),
        target_database_url: Some("postgresql://target/app".to_string()),
        source_id: "store-fleet".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication: "trellara_sales".to_string(),
        slot: "trellara_sales_slot".to_string(),
        capture: "pgoutput".to_string(),
        wal_retention_warn_bytes: None,
        table: vec!["public.sales".to_string()],
        write_init: Some(output.clone()),
        force: false,
    };
    let tables = vec![preflight_table_named("public", "sales")];
    let mut summary = DirectSourceSafetySummary::from_source_inspection(
        "store-fleet".to_string(),
        "sales".to_string(),
        None,
        tables.clone(),
        active_healthy_slot(),
        Vec::new(),
        source_safety_init_recommendation(&args, &tables),
    );

    assert!(matches!(
        maybe_write_source_safety_init_config(&args, &tables, &mut summary),
        Err(CliError::InvalidConfig(message)) if message.contains("already exists")
    ));

    fs::remove_dir_all(root).expect("remove write-init temp dir");
}

#[test]
fn source_safety_write_init_refuses_critical_source_blockers() {
    let output = PathBuf::from("trellara.yml");
    let args = SourceSafetyArgs {
        config: None,
        format: SourceSafetyOutputFormat::Json,
        output: None,
        database_url: Some("postgresql://source/app".to_string()),
        target_database_url: Some("postgresql://target/app".to_string()),
        source_id: "store-fleet".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication: "trellara_sales".to_string(),
        slot: "trellara_sales_slot".to_string(),
        capture: "pgoutput".to_string(),
        wal_retention_warn_bytes: None,
        table: vec!["public.sales".to_string()],
        write_init: Some(output),
        force: false,
    };
    let unsafe_table = trellara_pg_capture::TablePreflight {
        update_delete_safe: false,
        issues: vec!["table has no primary key".to_string()],
        ..preflight_table_named("public", "sales")
    };
    let tables = vec![unsafe_table];
    let mut summary = DirectSourceSafetySummary::from_source_inspection(
        "store-fleet".to_string(),
        "sales".to_string(),
        None,
        tables.clone(),
        active_healthy_slot(),
        Vec::new(),
        source_safety_init_recommendation(&args, &tables),
    );

    assert_eq!(summary.critical_factor_count, 1);
    assert!(matches!(
        maybe_write_source_safety_init_config(&args, &tables, &mut summary),
        Err(CliError::InvalidConfig(message)) if message.contains("critical source blockers")
    ));
    assert!(summary.init_config_written.is_none());
}
