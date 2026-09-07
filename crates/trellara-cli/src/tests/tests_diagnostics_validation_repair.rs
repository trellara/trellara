use super::*;

#[test]
fn repair_plan_surfaces_validation_drift_table_steps() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(latest_validation(false)),
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert!(plan.plan_required);
    assert_eq!(plan.step_count, 4);
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.action_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "validation_drift_reseed",
            "validation_drift_verify",
            "validation_drift_reseed",
            "validation_drift_verify",
        ]
    );
    assert_eq!(
        plan.steps[0].command,
        "trellara reseed --config examples/retail-fleet/strict.yml --table public.sales"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara verify --config examples/retail-fleet/strict.yml --table public.sales"
    );
    assert_eq!(
        plan.steps[2].command,
        "trellara reseed --config examples/retail-fleet/strict.yml --table public.refunds"
    );
    assert!(plan.steps[0]
        .reason
        .contains("relations: public.sales, public.refunds"));
}

#[test]
fn repair_plan_surfaces_validation_drift_flow_steps_without_relations() {
    let mut validation = latest_validation(false);
    validation.drift_relations.clear();
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(validation),
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert_eq!(plan.step_count, 2);
    assert_eq!(
        plan.steps[0].command,
        "trellara reseed --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
}

#[test]
fn repair_plan_surfaces_stale_validation_verify_step() {
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(validation),
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert!(plan.plan_required);
    assert_eq!(plan.step_count, 1);
    assert_eq!(plan.steps[0].action_code, "validation_freshness_verify");
    assert_eq!(
        plan.steps[0].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
    assert!(plan.steps[0]
        .reason
        .contains("validation watermarks are stale"));
    assert!(plan.steps[0].hint.contains("current checkpoint boundary"));
}

#[test]
fn repair_plan_surfaces_missing_partition_watermark_steps() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: None,
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/partitioned.yml"));

    assert!(plan.plan_required);
    assert_eq!(plan.step_count, 3);
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.action_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "partition_watermark_apply",
            "partition_watermark_inspect",
            "partition_watermark_verify",
        ]
    );
    assert_eq!(
        plan.steps[0].command,
        "trellara apply --config examples/retail-fleet/partitioned.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara partition-watermarks --config examples/retail-fleet/partitioned.yml"
    );
    assert!(plan.steps[0]
        .reason
        .contains("no partition watermark evidence"));
}

#[test]
fn repair_plan_surfaces_incomplete_partition_watermark_steps() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/partitioned.yml"));

    assert_eq!(plan.step_count, 3);
    assert!(plan.steps[0]
        .reason
        .contains("partition watermarks are missing 2 of 4 partitions"));
}

#[test]
fn repair_plan_surfaces_schema_handoff_steps() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let drift = source_schema_drift();
    let actions = flow_recovery_actions(&config, &healthy_slot(), Some(&drift), None)
        .expect("recovery actions");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_schema_drift: Some(drift),
        recovery_actions: actions,
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert!(plan.plan_required);
    assert_eq!(
        plan.latest_failure
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("source_schema_handoff_required")
    );
    assert_eq!(plan.step_count, 6);
    assert_eq!(
        plan.steps[0].command,
        "trellara schema-discover --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara contract-test --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[2].command,
        "trellara snapshot --config examples/retail-fleet/strict.yml --force"
    );
    assert_eq!(
        plan.steps[5].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
}
