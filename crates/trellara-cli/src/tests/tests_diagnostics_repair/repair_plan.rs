use super::*;

#[test]
fn repair_plan_is_empty_for_clean_flow() {
    let plan = RepairPlanSummary::from_status(
        clean_status(),
        Path::new("examples/retail-fleet/strict.yml"),
    );

    assert_eq!(plan.source_id, "source-a");
    assert_eq!(plan.dataset_id, "sales");
    assert!(plan.dry_run);
    assert!(!plan.plan_required);
    assert_eq!(plan.step_count, 0);
    assert!(plan.steps.is_empty());
    assert_eq!(plan.latest_failure, None);
}

#[test]
fn repair_plan_flattens_recovery_actions_with_config_path() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(latest_quarantine()),
        recovery_actions: vec![blocked_recovery_action()],
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert!(plan.plan_required);
    assert_eq!(plan.step_count, 1);
    assert_eq!(
        plan.latest_failure
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("target_quarantine_blocked")
    );
    assert_eq!(plan.steps[0].order, 1);
    assert_eq!(plan.steps[0].action_code, "target_quarantine_replay");
    assert_eq!(
        plan.steps[0].command,
        "trellara quarantine replay-ready --config examples/retail-fleet/strict.yml --transaction-id tx-blocked --commit-lsn 0/16B6D28"
    );
    assert_eq!(
        plan.steps[0].redelivery_topics,
        vec!["trellara.source-a.sales.strict".to_string()]
    );
    assert_eq!(
        plan.steps[0].redelivery_warnings,
        vec![
            "local redelivery boundary is incomplete".to_string(),
            "redelivery requires operator confirmation".to_string()
        ]
    );
}

#[test]
fn repair_plan_preserves_multi_step_source_reseed_order() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &lost_wal_slot(), None, None).expect("recovery actions");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_slot: lost_wal_slot(),
        recovery_actions: actions,
        ..clean_status_parts()
    });
    let plan =
        RepairPlanSummary::from_status(status, Path::new("examples/retail-fleet/strict.yml"));

    assert_eq!(plan.step_count, 5);
    assert_eq!(
        plan.steps[0].command,
        "trellara bootstrap --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara reseed --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[2].command,
        "trellara relay --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[3].command,
        "trellara apply --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[4].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
}
