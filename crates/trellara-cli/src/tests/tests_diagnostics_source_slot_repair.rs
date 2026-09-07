use super::*;

#[test]
fn source_slot_missing_recovery_action_bootstraps_fresh_handoff() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &missing_slot(), None, None).expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_slot_bootstrap_or_reseed");
    assert!(actions[0].reason.contains("does not exist"));
    assert_eq!(actions[0].transaction_id, None);
    assert_eq!(actions[0].commit_lsn, None);
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara bootstrap --config <config>".to_string(),
            "trellara snapshot --config <config> --force".to_string(),
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara status --config <config> --view alerts".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    );
    assert!(actions[0].redelivery_topics.is_empty());
    assert!(actions[0].hint.contains("source slot is missing"));
    assert!(actions[0].hint.contains("fresh snapshot-to-stream handoff"));
}

#[test]
fn repair_plan_surfaces_missing_slot_bootstrap_steps() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &missing_slot(), None, None).expect("recovery actions");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_slot: missing_slot(),
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
        Some("source_slot_missing")
    );
    assert_eq!(
        plan.latest_failure
            .as_ref()
            .map(|failure| failure.recovery_action_codes.clone()),
        Some(vec!["source_slot_bootstrap_or_reseed".to_string()])
    );
    assert_eq!(plan.step_count, 6);
    assert_eq!(
        plan.steps[0].command,
        "trellara bootstrap --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara snapshot --config examples/retail-fleet/strict.yml --force"
    );
    assert_eq!(
        plan.steps[4].command,
        "trellara status --config examples/retail-fleet/strict.yml --view alerts"
    );
    assert_eq!(
        plan.steps[5].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
}

#[test]
fn source_slot_wal_pressure_recovery_action_drains_before_reseed() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &unreserved_slot(), None, None).expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_slot_drain_before_wal_loss");
    assert!(actions[0].reason.contains("wal_status=unreserved"));
    assert_eq!(actions[0].transaction_id, None);
    assert_eq!(actions[0].commit_lsn, None);
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara status --config <config> --view alerts".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    );
    assert!(actions[0].hint.contains("still recoverable"));
    assert!(actions[0].hint.contains("mandatory reseed"));
}

#[test]
fn source_schema_drift_recovery_action_requires_fresh_handoff() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let drift = source_schema_drift();
    let actions = flow_recovery_actions(&config, &healthy_slot(), Some(&drift), None)
        .expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_schema_handoff_required");
    assert!(actions[0].reason.contains("pgoutput metadata"));
    assert!(actions[0].reason.contains("public.sales"));
    assert_eq!(actions[0].transaction_id, None);
    assert_eq!(actions[0].commit_lsn, None);
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara schema-discover --config <config>".to_string(),
            "trellara contract-test --config <config>".to_string(),
            "trellara snapshot --config <config> --force".to_string(),
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    );
    assert!(actions[0]
        .hint
        .contains("fresh audited snapshot-to-stream handoff"));
}

#[test]
fn repair_plan_surfaces_pre_loss_slot_drain_steps() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &unreserved_slot(), None, None).expect("recovery actions");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_slot: unreserved_slot(),
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
        Some("source_slot_wal_unreserved")
    );
    assert_eq!(plan.step_count, 4);
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.action_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "source_slot_drain_before_wal_loss",
            "source_slot_drain_before_wal_loss",
            "source_slot_drain_before_wal_loss",
            "source_slot_drain_before_wal_loss",
        ]
    );
    assert_eq!(
        plan.steps[0].command,
        "trellara relay --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara apply --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[2].command,
        "trellara status --config examples/retail-fleet/strict.yml --view alerts"
    );
    assert_eq!(
        plan.steps[3].command,
        "trellara verify --config examples/retail-fleet/strict.yml"
    );
}
