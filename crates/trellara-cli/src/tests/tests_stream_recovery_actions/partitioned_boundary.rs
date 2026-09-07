use super::*;

#[test]
fn quarantine_recovery_action_preserves_partitioned_boundary_topics() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 2\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let actions = flow_recovery_actions(&config, &healthy_slot(), None, Some(&latest_quarantine()))
        .expect("recovery actions");

    assert_eq!(
        actions[0].redelivery_topics,
        vec![
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
            "trellara.local-source.retail-sales.partition.0".to_string(),
            "trellara.local-source.retail-sales.partition.1".to_string(),
        ]
    );
    assert!(actions[0]
        .hint
        .contains("manifest, commit marker, and participating partition messages"));
    assert!(actions[0].hint.contains("barrier-aware"));
}

#[tokio::test]
async fn quarantine_recovery_action_warns_for_incomplete_partitioned_boundary() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-recovery-action-partitioned-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml()
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 2\n    key_column: store_id",
        )
        .replace(
            "  path: /tmp/trellara-local-stream",
            &format!("  path: {}", root.display()),
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_partitioned_transaction(&config, "tx-blocked", true).await;
    let quarantine = ApplyQuarantine {
        commit_lsn: "0/16B6C90".to_string(),
        ..latest_quarantine()
    };

    let actions = flow_recovery_actions(&config, &healthy_slot(), None, Some(&quarantine))
        .expect("recovery actions");

    assert_eq!(
        actions[0].redelivery_warnings,
        vec![
            "local redelivery boundary is incomplete_boundary; exact seek commands were not generated"
                .to_string(),
            "missing message kinds: partition_chunk".to_string()
        ]
    );
    assert!(actions[0].command_templates.iter().any(|command| {
        command.contains("trellara.local-source.retail-sales.manifest")
            && command.contains("--next-offset <offset>")
    }));

    std::fs::remove_dir_all(root).expect("cleanup");
}
