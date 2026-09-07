use super::*;

#[test]
fn pilot_package_ddl_status_rejects_duplicate_required_sinks() {
    let (config, mut summary, _) = partitioned_release_inputs();
    let duplicate = summary.propagation.sinks[0].clone();
    summary.propagation.sinks.push(duplicate);

    let error = pilot_package_ddl_barrier_status(&config, &summary).expect_err("invalid summary");

    assert!(error.to_string().contains("duplicate required sinks"));
}

#[test]
fn pilot_package_ddl_release_rejects_duplicate_required_sinks() {
    let (config, mut summary, apply_plan) = partitioned_release_inputs();
    let duplicate = summary.propagation.sinks[0].clone();
    summary.propagation.sinks.push(duplicate);

    let error = pilot_package_ddl_release_proof(&config, &summary, &apply_plan)
        .expect_err("invalid release proof");

    assert!(error.to_string().contains("duplicate required sinks"));
}

fn partitioned_release_inputs() -> (TrellaraConfig, DdlPlanSummary, DdlApplyPlanSummary) {
    let yaml = STRICT_YAML.replace(
        "  mode: strict_transaction_order",
        "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = DdlPlanArgs {
        config: PathBuf::from("partitioned.yml"),
        changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
        apply_mode: DdlPlanApplyMode::AutoSafe,
        format: QuickstartOutputFormat::Json,
    };
    let summary = DdlPlanSummary::from_args(&config, &args).expect("ddl plan");
    let apply_plan = DdlApplyPlanSummary::from_args(&config, &args).expect("ddl apply plan");

    (config, summary, apply_plan)
}
