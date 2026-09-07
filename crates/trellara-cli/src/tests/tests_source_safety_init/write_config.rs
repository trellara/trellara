use super::*;

#[test]
fn source_safety_write_init_materializes_evaluation_config() {
    let root = std::env::temp_dir().join(format!(
        "trellara-source-safety-write-init-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let output = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create write-init temp dir");
    let args = SourceSafetyArgs {
        config: None,
        format: SourceSafetyOutputFormat::Text,
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
        table: vec!["public.sales".to_string(), "public.payments".to_string()],
        write_init: Some(output.clone()),
        force: false,
    };
    let mut payments = preflight_table_named("public", "payments");
    payments.primary_key_columns = vec!["payment_id".to_string()];
    payments.columns[0].name = "payment_id".to_string();
    payments.columns[0].is_key = true;
    payments.schema_fingerprint = Some(84);
    let tables = vec![preflight_table_named("public", "sales"), payments];
    let mut summary = DirectSourceSafetySummary::from_source_inspection(
        "store-fleet".to_string(),
        "sales".to_string(),
        None,
        tables.clone(),
        active_healthy_slot(),
        Vec::new(),
        source_safety_init_recommendation(&args, &tables),
    );

    maybe_write_source_safety_init_config(&args, &tables, &mut summary).expect("write init");

    assert_eq!(
        summary.init_config_written.as_deref(),
        Some(output.to_str().unwrap())
    );
    let recommendation = summary
        .init_recommendation
        .as_ref()
        .expect("init recommendation");
    assert_eq!(recommendation.output, output.display().to_string());
    assert!(recommendation
        .command
        .contains("--target-database-url postgresql://target/app"));
    assert!(recommendation.command.contains("--evaluate"));
    let generated = TrellaraConfig::from_path(&output).expect("generated config");
    generated.validate().expect("valid generated config");
    assert_eq!(generated.source.id, "store-fleet");
    assert_eq!(generated.dataset.id, "sales");
    assert_eq!(generated.status_mode(), "strict_chunked_transaction_order");
    assert_eq!(generated.dataset.tables.len(), 2);
    assert_eq!(
        generated.dataset.tables[0]
            .verify
            .as_ref()
            .map(|verify| verify.primary_key.as_str()),
        Some("id")
    );
    assert_eq!(
        generated.dataset.tables[0]
            .contract
            .as_ref()
            .and_then(|contract| contract.source_schema_fingerprint),
        Some(42)
    );
    assert_eq!(generated.dataset.tables[1].name, "payments");
    assert_eq!(
        generated.dataset.tables[1]
            .verify
            .as_ref()
            .map(|verify| verify.primary_key.as_str()),
        Some("payment_id")
    );
    assert_eq!(
        generated.dataset.tables[1]
            .contract
            .as_ref()
            .and_then(|contract| contract.source_schema_fingerprint),
        Some(84)
    );
    assert_eq!(
        generated
            .target
            .as_ref()
            .map(|target| target.database_url.as_str()),
        Some("postgresql://target/app")
    );

    fs::remove_dir_all(root).expect("remove write-init temp dir");
}
