use super::*;

#[test]
fn ddl_barrier_record_uses_plan_sink_contract() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    let barrier = ddl_barrier_from_plan(&config, &plan, "0/16B8000", "schema-v2").expect("barrier");

    assert_eq!(barrier.source_id, "local-source");
    assert_eq!(barrier.dataset_id, "retail-sales");
    assert_eq!(barrier.barrier_id, plan.propagation.barrier_id);
    assert_eq!(barrier.barrier_lsn, "0/16B8000");
    assert_eq!(barrier.schema_version, "schema-v2");
    assert_eq!(
        barrier.cdc_transaction_boundary,
        plan.propagation.cdc_transaction_boundary
    );
    assert_eq!(
        barrier.required_sinks,
        vec![
            "target_postgres".to_string(),
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string()
        ]
    );
    assert!(!barrier.requires_global_partition_pause);
}

#[test]
fn ddl_barrier_record_preserves_partition_visibility_sink() {
    let yaml = STRICT_YAML.replace(
            "  mode: strict_transaction_order",
            "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let plan = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    let barrier = ddl_barrier_from_plan(&config, &plan, "0/16B8000", "schema-v2").expect("barrier");

    assert!(barrier.requires_global_partition_pause);
    assert!(barrier
        .required_sinks
        .contains(&"partition_visibility".to_string()));
}

#[test]
fn ddl_barrier_record_normalizes_plan_sinks_before_recording() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let mut plan = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");
    plan.propagation
        .sinks
        .retain(|sink| sink.name != "target_postgres");
    plan.propagation.sinks.push(DdlPropagationSink {
        name: "raw_cdc_lake".to_string(),
        kind: DdlPropagationSinkKind::RawCdcLake,
        required_ack: "duplicate raw ack".to_string(),
        ack_evidence: "duplicate raw evidence".to_string(),
    });

    let barrier = ddl_barrier_from_plan(&config, &plan, "0/16B8000", "schema-v2").expect("barrier");

    assert_eq!(
        barrier.required_sinks,
        vec![
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string(),
            "target_postgres".to_string()
        ]
    );
}

#[test]
fn ddl_barrier_record_restores_partition_visibility_when_pause_required() {
    let yaml = STRICT_YAML.replace(
            "  mode: strict_transaction_order",
            "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let mut plan = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");
    plan.propagation
        .sinks
        .retain(|sink| sink.name != "partition_visibility");

    let barrier = ddl_barrier_from_plan(&config, &plan, "0/16B8000", "schema-v2").expect("barrier");

    assert_eq!(
        barrier.required_sinks.last().map(String::as_str),
        Some("partition_visibility")
    );
    assert!(barrier.requires_global_partition_pause);
}

#[test]
fn ddl_barrier_record_rejects_invalid_barrier_lsn() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = strict_plan(&config);

    for barrier_lsn in ["", "   ", " 0/16B8000", "0/0", "not-a-lsn"] {
        let error =
            ddl_barrier_from_plan(&config, &plan, barrier_lsn, "schema-v2").expect_err("bad lsn");

        assert!(
            error.to_string().contains("--barrier-lsn"),
            "unexpected error for {barrier_lsn:?}: {error}"
        );
    }
}

#[test]
fn ddl_barrier_record_rejects_invalid_schema_version() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = strict_plan(&config);

    for schema_version in ["", "   ", " schema-v2", "schema-v2 "] {
        let error = ddl_barrier_from_plan(&config, &plan, "0/16B8000", schema_version)
            .expect_err("bad schema version");

        assert!(
            error.to_string().contains("--schema-version"),
            "unexpected error for {schema_version:?}: {error}"
        );
    }
}

fn strict_plan(config: &TrellaraConfig) -> DdlPlanSummary {
    DdlPlanSummary::from_args(
        config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan")
}
