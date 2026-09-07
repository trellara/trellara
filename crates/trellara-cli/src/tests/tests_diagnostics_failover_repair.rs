use super::*;

#[test]
fn source_slot_failover_disabled_recovery_action_prepares_promotion() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions = flow_recovery_actions(&config, &failover_disabled_slot(), None, None)
        .expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_slot_enable_failover_slot");
    assert!(actions[0]
        .reason
        .contains("not configured as a failover slot"));
    assert_eq!(actions[0].transaction_id, None);
    assert_eq!(actions[0].commit_lsn, None);
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara check --config <config>".to_string(),
            "trellara status --config <config> --view alerts".to_string(),
        ]
    );
    assert!(actions[0].redelivery_topics.is_empty());
    assert!(actions[0].hint.contains("enable a failover logical slot"));
}

#[test]
fn source_slot_failover_unsynced_recovery_action_waits_for_sync() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions = flow_recovery_actions(&config, &failover_unsynced_slot(), None, None)
        .expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_slot_wait_for_failover_sync");
    assert!(actions[0].reason.contains("not synced to the standby"));
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara check --config <config>".to_string(),
            "trellara status --config <config> --view alerts".to_string(),
        ]
    );
    assert!(actions[0]
        .hint
        .contains("repair standby slot synchronization"));
}

#[test]
fn repair_plan_surfaces_failover_slot_readiness_steps() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions = flow_recovery_actions(&config, &failover_unsynced_slot(), None, None)
        .expect("recovery actions");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_slot: failover_unsynced_slot(),
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
        Some("source_slot_failover_not_synced")
    );
    assert_eq!(plan.step_count, 2);
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.action_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "source_slot_wait_for_failover_sync",
            "source_slot_wait_for_failover_sync",
        ]
    );
    assert_eq!(
        plan.steps[0].command,
        "trellara check --config examples/retail-fleet/strict.yml"
    );
    assert_eq!(
        plan.steps[1].command,
        "trellara status --config examples/retail-fleet/strict.yml --view alerts"
    );
}
