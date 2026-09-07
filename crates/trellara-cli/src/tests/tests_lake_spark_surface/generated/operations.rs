use super::*;

#[test]
fn lake_spark_maintenance_template_renders_without_primary_key() {
    let yaml = STRICT_YAML.replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n      verify:\n        primary_key: \"\"",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let mut args = spark_template_args("epoch-1", true);
    args.target_table = Some("retail_sales__public__sales__scd2".to_string());
    let summary =
        LakeSparkTemplateSummary::from_args(&config, &args, LakeSparkTemplateKind::Maintenance)
            .expect("maintenance template");

    assert_eq!(summary.template, LakeSparkTemplateKind::Maintenance);
    assert_eq!(summary.target_table, "retail_sales__public__sales__scd2");
    assert_eq!(
        summary.template_sha256,
        lake_spark_template_sha256(&summary.sql)
    );
    assert!(summary
        .sql
        .contains("CALL spark_catalog.system.rewrite_data_files"));
    assert!(summary
        .sql
        .contains("CALL spark_catalog.system.expire_snapshots"));
    assert!(summary
        .sql
        .contains("retail_sales__trellara__fanin___trellara_verification"));
    assert!(summary.sql.contains("v.checksum_status = 'match'"));
    assert!(summary
        .sql
        .contains("raise_error('Trellara epoch is not consumable"));
    assert_policy_guarded_gap_gate(&summary.sql, "true");
    assert!(summary.pyspark_runner.contains("spark-maintenance.sql"));
    assert!(summary.pyspark_runner.contains("template=maintenance"));
    assert!(summary.pyspark_runner.contains("template_sha256="));
    assert!(summary
        .pyspark_runner
        .contains("accept_complete_with_gaps=true"));
    assert!(!summary.pyspark_runner.contains("${"));
    assert!(summary
        .idempotency_rule
        .contains("metadata/file-layout only"));
    assert!(!summary.sql.contains("${"));
}

#[test]
fn lake_spark_dashboard_template_renders_completeness_queries_without_primary_key() {
    let yaml = STRICT_YAML.replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n      verify:\n        primary_key: \"\"",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = LakeSparkTemplateSummary::from_args(
        &config,
        &spark_template_args("epoch-1", true),
        LakeSparkTemplateKind::Dashboard,
    )
    .expect("dashboard template");

    assert_eq!(summary.template, LakeSparkTemplateKind::Dashboard);
    assert_eq!(
        summary.target_table,
        "retail_sales__trellara__fanin___trellara_epoch_sources"
    );
    assert_eq!(
        summary.quarantine_table,
        "retail_sales__trellara__fanin___trellara_quarantine"
    );
    assert_eq!(
        summary.epoch_partitions_table,
        "retail_sales__trellara__fanin___trellara_epoch_partitions"
    );
    assert_eq!(
        summary.template_sha256,
        lake_spark_template_sha256(&summary.sql)
    );
    assert!(summary.sql.contains("epoch_release_gate"));
    assert!(summary.sql.contains("partition_evidence"));
    assert!(summary.sql.contains("safe_to_publish"));
    assert!(summary.sql.contains(
        "FROM spark_catalog.retail_sales.retail_sales__trellara__fanin___trellara_epochs"
    ));
    assert!(summary.sql.contains(
        "FROM spark_catalog.retail_sales.retail_sales__trellara__fanin___trellara_epoch_sources"
    ));
    assert!(summary.sql.contains(
        "FROM spark_catalog.retail_sales.retail_sales__trellara__fanin___trellara_epoch_partitions"
    ));
    assert!(summary.sql.contains(
        "FROM spark_catalog.retail_sales.retail_sales__trellara__fanin___trellara_quarantine"
    ));
    assert!(summary.sql.contains("partition_event_count"));
    assert!(summary.sql.contains("recovery_command"));
    assert!(summary.sql.contains("v.checksum_status = 'match'"));
    assert_policy_guarded_gap_gate(&summary.sql, "true");
    assert!(summary
        .sql
        .contains("raise_error('Trellara epoch is not consumable"));
    assert!(summary
        .pyspark_runner
        .contains("spark-completeness-dashboard.sql"));
    assert!(summary.pyspark_runner.contains("template=dashboard"));
    assert!(summary.pyspark_runner.contains("template_sha256="));
    assert!(summary
        .pyspark_runner
        .contains("accept_complete_with_gaps=true"));
    assert!(summary.idempotency_rule.contains("read-only completeness"));
    assert!(!summary.sql.contains("${"));
    assert!(!summary.pyspark_runner.contains("${"));
}
