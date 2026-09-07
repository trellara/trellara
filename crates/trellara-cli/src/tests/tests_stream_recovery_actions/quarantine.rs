use super::*;

#[test]
fn quarantine_recovery_action_includes_command_and_strict_topics() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");
    let actions = flow_recovery_actions(&config, &healthy_slot(), None, Some(&latest_quarantine()))
        .expect("recovery actions");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].code, "target_quarantine_replay");
    assert_eq!(actions[0].reason, "target_postgres_error");
    assert_eq!(actions[0].transaction_id.as_deref(), Some("tx-blocked"));
    assert_eq!(actions[0].commit_lsn.as_deref(), Some("0/16B6D28"));
    assert_eq!(
        actions[0].command_templates,
        vec![
            "trellara quarantine replay-ready --config <config> --transaction-id tx-blocked --commit-lsn 0/16B6D28"
                .to_string(),
            "trellara stream locate-local --config <config> --transaction-id tx-blocked --commit-lsn 0/16B6D28"
                .to_string(),
            "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset <offset>"
                .to_string()
        ]
    );
    assert_eq!(
        actions[0].redelivery_topics,
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
    assert_eq!(
        actions[0].redelivery_warnings,
        vec![
            "local redelivery boundary is not_found; exact seek commands were not generated"
                .to_string(),
            "missing message kinds: strict_transaction".to_string()
        ]
    );
    assert!(actions[0]
        .command_templates
        .iter()
        .any(|command| command.contains("stream locate-local")));
    assert!(actions[0]
        .command_templates
        .iter()
        .any(|command| command.contains("stream seek-local")));
    assert!(actions[0].hint.contains("strict transaction"));
}

#[test]
fn quarantine_recovery_action_explains_zero_row_match() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let quarantine = ApplyQuarantine {
        reason: "no_rows_matched".to_string(),
        ..latest_quarantine()
    };
    let actions = flow_recovery_actions(&config, &healthy_slot(), None, Some(&quarantine))
        .expect("recovery actions");

    assert_eq!(actions[0].reason, "no_rows_matched");
    assert!(actions[0]
        .hint
        .contains("target row was missing or diverged"));
    assert!(actions[0].hint.contains("trellara verify"));
    assert!(actions[0].hint.contains("reseed"));
    assert!(actions[0].hint.contains("replay-ready"));
}

#[test]
fn quarantine_recovery_action_preserves_strict_chunk_boundary_topics() {
    let yaml = STRICT_YAML.replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 2",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let actions = flow_recovery_actions(&config, &healthy_slot(), None, Some(&latest_quarantine()))
        .expect("recovery actions");

    assert_eq!(
        actions[0].redelivery_topics,
        vec![
            "trellara.local-source.retail-sales.strict".to_string(),
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
        ]
    );
    assert!(actions[0]
        .hint
        .contains("strict chunk messages plus manifest and commit marker barrier"));
    assert!(actions[0]
        .hint
        .contains("reconstruct the complete transaction before apply"));
}
