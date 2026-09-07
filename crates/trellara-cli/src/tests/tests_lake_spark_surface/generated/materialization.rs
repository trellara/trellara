use super::*;

#[test]
fn lake_spark_current_state_template_renders_configured_sql() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakeSparkTemplateSummary::from_args(
        &config,
        &spark_template_args("epoch-1", true),
        LakeSparkTemplateKind::CurrentState,
    )
    .expect("spark template");

    assert_eq!(summary.template, LakeSparkTemplateKind::CurrentState);
    assert_eq!(summary.namespace, "retail_sales");
    assert_eq!(
        summary.raw_cdc_table,
        "retail_sales__public__sales__raw_cdc"
    );
    assert_eq!(
        summary.epochs_table,
        "retail_sales__trellara__fanin___trellara_epochs"
    );
    assert_eq!(
        summary.verification_table,
        "retail_sales__trellara__fanin___trellara_verification"
    );
    assert_eq!(
        summary.epoch_partitions_table,
        "retail_sales__trellara__fanin___trellara_epoch_partitions"
    );
    assert_eq!(summary.target_table, "retail_sales__public__sales__current");
    assert_eq!(summary.primary_key_column, "id");
    assert_eq!(
        summary.template_sha256,
        lake_spark_template_sha256(&summary.sql)
    );
    assert_eq!(summary.template_sha256.len(), 64);
    assert!(summary
        .sql
        .contains("MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__current"));
    assert!(summary
        .sql
        .contains("FROM spark_catalog.retail_sales.retail_sales__public__sales__raw_cdc"));
    assert_policy_guarded_gap_gate(&summary.sql, "true");
    assert!(summary.sql.contains(
        "JOIN spark_catalog.retail_sales.retail_sales__trellara__fanin___trellara_verification"
    ));
    assert!(summary.sql.contains("v.checksum_status = 'match'"));
    assert!(summary
        .sql
        .contains("raise_error('Trellara epoch is not consumable"));
    assert!(summary.sql.contains("trellara_epoch_consumable"));
    assert!(summary.pyspark_runner.contains("spark-current-state.sql"));
    assert!(summary.pyspark_runner.contains("template_sha256="));
    assert!(summary
        .pyspark_runner
        .contains("refusing unresolved Trellara Spark template placeholders"));
    assert!(summary.pyspark_runner.contains("epoch_id=epoch-1"));
    assert!(summary
        .pyspark_runner
        .contains("accept_complete_with_gaps=true"));
    assert!(!summary.pyspark_runner.contains("${"));
    assert!(summary
        .sql
        .contains("c.record_key AS __trellara_record_key"));
    assert!(summary
        .sql
        .contains("c.payload_after_json AS row_after_json"));
    assert!(summary
        .sql
        .contains("c.commit_lsn AS __trellara_commit_lsn"));
    assert!(summary
        .sql
        .contains("timestamp_millis(c.commit_timestamp_ms)"));
    assert!(summary
        .visibility_rule
        .contains("verification checksum_status is match"));
    assert!(!summary.sql.contains("${"));
}

#[test]
fn lake_spark_scd2_template_renders_valid_time_sql() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut args = spark_template_args("epoch-2", false);
    args.catalog = "demo_catalog".to_string();
    args.namespace = Some("analytics".to_string());
    args.target_table = Some("sales_history".to_string());
    args.primary_key_column = Some("sale_id".to_string());
    args.format = QuickstartOutputFormat::Json;
    let summary = LakeSparkTemplateSummary::from_args(&config, &args, LakeSparkTemplateKind::Scd2)
        .expect("spark template");

    assert_eq!(summary.template, LakeSparkTemplateKind::Scd2);
    assert_eq!(summary.namespace, "analytics");
    assert_eq!(summary.target_table, "sales_history");
    assert_eq!(summary.primary_key_column, "sale_id");
    assert_eq!(
        summary.template_sha256,
        lake_spark_template_sha256(&summary.sql)
    );
    assert!(summary
        .sql
        .contains("MERGE INTO demo_catalog.analytics.sales_history"));
    assert!(summary.sql.contains("WHEN NOT MATCHED THEN INSERT"));
    assert!(summary.sql.contains("__trellara_valid_from"));
    assert_policy_guarded_gap_gate(&summary.sql, "false");
    assert!(summary.sql.contains("v.checksum_status = 'match'"));
    assert!(summary
        .sql
        .contains("raise_error('Trellara epoch is not consumable"));
    assert!(summary.sql.contains("trellara_epoch_consumable"));
    assert!(summary.pyspark_runner.contains("spark-scd2.sql"));
    assert!(summary.pyspark_runner.contains("template=scd2"));
    assert!(summary.pyspark_runner.contains("template_sha256="));
    assert!(summary
        .pyspark_runner
        .contains("accept_complete_with_gaps=false"));
    assert!(!summary.pyspark_runner.contains("${"));
    assert!(summary
        .sql
        .contains("c.record_key AS __trellara_record_key"));
    assert!(summary
        .sql
        .contains("c.payload_after_json AS row_after_json"));
    assert!(summary
        .sql
        .contains("c.commit_lsn AS __trellara_commit_lsn"));
    assert!(summary
        .sql
        .contains("timestamp_millis(c.commit_timestamp_ms)"));
    assert!(!summary.sql.contains("${"));
}

#[test]
fn lake_spark_template_requires_primary_key_or_override() {
    let yaml = STRICT_YAML.replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n      verify:\n        primary_key: \"\"",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let error = LakeSparkTemplateSummary::from_args(
        &config,
        &spark_template_args("epoch-1", false),
        LakeSparkTemplateKind::CurrentState,
    )
    .expect_err("primary key required");

    assert!(matches!(
        error,
        CliError::InvalidConfig(message)
            if message.contains("requires dataset.tables[].verify.primary_key")
    ));
}

#[test]
fn lake_spark_template_requires_reason_for_unsafe_epoch_override() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut args = spark_template_args("epoch-unsafe", false);
    args.unsafe_allow_non_consumable_epoch = true;

    let error =
        LakeSparkTemplateSummary::from_args(&config, &args, LakeSparkTemplateKind::CurrentState)
            .expect_err("unsafe override reason required");

    assert!(matches!(
        error,
        CliError::InvalidConfig(message)
            if message.contains("requires --unsafe-override-reason")
    ));
}

#[test]
fn lake_spark_template_records_unsafe_epoch_override_reason() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut args = spark_template_args("epoch-unsafe", false);
    args.unsafe_allow_non_consumable_epoch = true;
    args.unsafe_override_reason = Some("incident replay window".to_string());
    let summary =
        LakeSparkTemplateSummary::from_args(&config, &args, LakeSparkTemplateKind::CurrentState)
            .expect("unsafe override template");

    assert!(summary.unsafe_allow_non_consumable_epoch);
    assert_eq!(
        summary.unsafe_override_reason.as_deref(),
        Some("incident replay window")
    );
    assert!(summary.visibility_rule.contains("unsafe override enabled"));
    assert!(summary.sql.contains("OR true = true"));
    assert!(summary.sql.contains("incident replay window"));
    assert!(summary
        .pyspark_runner
        .contains("unsafe_allow_non_consumable_epoch=true"));
}

#[test]
fn lake_spark_template_normalizes_unsafe_epoch_override_reason() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut args = spark_template_args("epoch-unsafe", false);
    args.unsafe_allow_non_consumable_epoch = true;
    args.unsafe_override_reason = Some("  incident replay window  ".to_string());
    let summary =
        LakeSparkTemplateSummary::from_args(&config, &args, LakeSparkTemplateKind::CurrentState)
            .expect("unsafe override template");

    assert_eq!(
        summary.unsafe_override_reason.as_deref(),
        Some("incident replay window")
    );
    assert!(summary.visibility_rule.contains("`incident replay window`"));
    assert!(!summary.visibility_rule.contains("  incident replay"));
    assert!(summary.sql.contains("incident replay window"));
    assert!(!summary.sql.contains("'  incident replay window  '"));
    assert!(summary
        .pyspark_runner
        .contains("unsafe_override_reason=incident replay window"));
    assert!(!summary.pyspark_runner.contains("  incident replay"));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("--unsafe-override-reason 'incident replay window'")));
    assert!(!summary
        .next_commands
        .iter()
        .any(|command| command.contains("  incident replay")));
}
