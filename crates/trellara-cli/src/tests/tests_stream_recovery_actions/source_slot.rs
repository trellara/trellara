use super::*;

#[test]
fn source_slot_wal_loss_recovery_action_reseeds_before_resume() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let actions =
        flow_recovery_actions(&config, &lost_wal_slot(), None, None).expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "source_slot_recreate_and_reseed");
    assert!(actions[0].reason.contains("invalidated"));
    assert_eq!(actions[0].transaction_id, None);
    assert_eq!(actions[0].commit_lsn, None);
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara bootstrap --config <config>".to_string(),
            "trellara reseed --config <config>".to_string(),
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    );
    assert!(actions[0].redelivery_topics.is_empty());
    assert!(actions[0].redelivery_warnings.is_empty());
    assert!(actions[0].hint.contains("fresh snapshot handoff"));
}
