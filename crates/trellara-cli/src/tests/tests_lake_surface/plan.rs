use super::*;

#[test]
fn lake_plan_summarizes_fanin_materializations_for_strict_flow() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = LakePlanSummary::from_config(&config);

    assert_eq!(summary.source_id, "local-source");
    assert_eq!(summary.dataset_id, "retail-sales");
    assert_eq!(summary.mode, "strict_transaction_order");
    assert_eq!(
        summary.contract,
        "fleet_fanin_append_only_raw_cdc_with_epoch_completeness"
    );
    assert_eq!(summary.fanin_mode, "strict_envelope_epoch_fanin");
    assert_eq!(summary.materialization_count, 4);
    assert_eq!(
        summary
            .materializations
            .iter()
            .map(|materialization| materialization.kind.as_str())
            .collect::<Vec<_>>(),
        vec![
            "raw_cdc_append_only",
            "epoch_metadata",
            "current_state_template",
            "scd2_history_template"
        ]
    );
    assert!(summary
        .materializations
        .iter()
        .any(|materialization| materialization.producer == "spark_template"));
    assert_eq!(
        summary.epoch_metadata_tables,
        vec![
            "_trellara_epochs",
            "_trellara_epoch_sources",
            "_trellara_epoch_tables",
            "_trellara_epoch_partitions",
            "_trellara_quarantine",
            "_trellara_verification"
        ]
    );
    assert_eq!(
        summary.spark_template_outputs,
        vec![
            "retail_sales__spark__derived__current_state",
            "retail_sales__spark__derived__scd2_history"
        ]
    );
    assert_eq!(summary.table_count, 1);
    assert_eq!(summary.tables[0].relation, "public.sales");
    assert_eq!(summary.tables[0].primary_key, "id");
    assert_eq!(
        summary.tables[0].raw_cdc_table,
        "retail_sales__public__sales__raw_cdc"
    );
    assert_eq!(summary.warning_check_count, 1);
    assert_eq!(summary.blocking_check_count, 0);
    assert!(summary.checks.iter().any(|check| {
        check.name == "transaction_boundary"
            && check.status == LakePlanCheckStatus::Ready
            && check
                .message
                .contains("complete source transaction envelope")
    }));
    assert!(summary.checks.iter().any(|check| {
        check.name == "schema_fingerprints" && check.status == LakePlanCheckStatus::Warning
    }));
}

#[test]
fn lake_plan_uses_pinned_schema_and_table_contracts() {
    let yaml = STRICT_YAML.replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      verify:\n        primary_key: sale_id\n        excluded_columns: [updated_at]\n        row_filter: \"region = 'west'\"\n      contract:\n        source_schema_fingerprint: 42\n        target_owned_columns: [reviewed_by]",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = LakePlanSummary::from_config(&config);

    assert_eq!(summary.warning_check_count, 0);
    assert_eq!(summary.blocking_check_count, 0);
    assert_eq!(summary.tables[0].primary_key, "sale_id");
    assert_eq!(summary.tables[0].source_schema_fingerprint, Some(42));
    assert_eq!(summary.tables[0].excluded_columns, vec!["updated_at"]);
    assert_eq!(
        summary.tables[0].row_filter.as_deref(),
        Some("region = 'west'")
    );
    assert_eq!(summary.tables[0].target_owned_columns, vec!["reviewed_by"]);
    assert!(summary.checks.iter().any(|check| {
        check.name == "schema_fingerprints" && check.status == LakePlanCheckStatus::Ready
    }));
}

#[test]
fn lake_plan_warns_spark_templates_without_primary_key() {
    let yaml = STRICT_YAML.replace(
        "    - schema: public\n      name: sales",
        "    - schema: public\n      name: sales\n      verify:\n        primary_key: ''",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = LakePlanSummary::from_config(&config);

    assert_eq!(summary.blocking_check_count, 0);
    assert_eq!(summary.warning_check_count, 2);
    assert!(summary
        .checks
        .iter()
        .any(|check| check.name == "primary_keys"
            && check.status == LakePlanCheckStatus::Warning
            && check
                .recommendation
                .as_deref()
                .is_some_and(|recommendation| recommendation.contains("Spark current-state"))));
}

#[test]
fn lake_plan_explains_strict_chunk_manifest_visibility_boundary() {
    let config = TrellaraConfig::from_yaml(&local_strict_chunking_yaml(), "test").expect("parse");
    let summary = LakePlanSummary::from_config(&config);

    assert!(summary
        .materializations
        .iter()
        .all(|materialization| materialization
            .visibility_boundary
            .contains("strict chunk manifest and commit marker barrier")));
    assert!(summary.checks.iter().any(|check| {
        check.name == "transaction_boundary"
            && check
                .message
                .contains("every chunk, the manifest, and the commit marker")
            && check
                .recommendation
                .as_deref()
                .is_some_and(|recommendation| {
                    recommendation.contains("strict chunk manifest and commit marker")
                })
    }));
}

#[test]
fn lake_plan_explains_partition_manifest_visibility_boundary() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = LakePlanSummary::from_config(&config);

    assert!(summary
        .materializations
        .iter()
        .all(|materialization| materialization
            .visibility_boundary
            .contains("manifest and commit marker barrier plus global low watermark")));
    assert!(summary.checks.iter().any(|check| {
        check.name == "transaction_boundary"
            && check.message.contains("manifest and commit marker barrier")
            && check
                .recommendation
                .as_deref()
                .is_some_and(|recommendation| recommendation.contains("partition-watermarks"))
    }));
}
