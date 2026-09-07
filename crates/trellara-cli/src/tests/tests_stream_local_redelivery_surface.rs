use super::*;

#[tokio::test]
async fn replay_redelivery_plan_resolves_exact_local_seek_commands() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-redelivery-plan-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 3).await;
    let topics = config
        .replay_redelivery_topics()
        .expect("redelivery topics");

    let plan = replay_redelivery_plan(&config, &topics, Some(("tx-local-seek-1", "0/16B6C51")))
        .expect("redelivery plan");

    assert_eq!(plan.boundary.as_ref().expect("boundary").status, "complete");
    assert_eq!(plan.warnings, Vec::<String>::new());
    assert_eq!(
        plan.exact_seek_commands,
        vec![
            "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset 1 --transaction-id tx-local-seek-1 --commit-lsn 0/16B6C51"
                .to_string()
        ]
    );
    assert_eq!(
            plan.commands,
            vec![
                "trellara stream locate-local --config <config> --transaction-id tx-local-seek-1 --commit-lsn 0/16B6C51"
                    .to_string(),
                "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset 1 --transaction-id tx-local-seek-1 --commit-lsn 0/16B6C51"
                    .to_string(),
            ]
        );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn replay_redelivery_plan_falls_back_when_local_boundary_is_absent() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-redelivery-plan-missing-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let topics = config
        .replay_redelivery_topics()
        .expect("redelivery topics");

    let plan = replay_redelivery_plan(&config, &topics, Some(("tx-missing", "0/16B6C51")))
        .expect("redelivery plan");

    assert_eq!(
        plan.boundary.as_ref().expect("boundary").status,
        "not_found"
    );
    assert_eq!(
        plan.warnings,
        vec![
            "local redelivery boundary is not_found; exact seek commands were not generated"
                .to_string(),
            "missing message kinds: strict_transaction".to_string()
        ]
    );
    assert!(plan.exact_seek_commands.is_empty());
    assert_eq!(
            plan.commands,
            vec![
                "trellara stream locate-local --config <config> --transaction-id tx-missing --commit-lsn 0/16B6C51"
                    .to_string(),
                "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset <offset>"
                    .to_string(),
            ]
        );

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn replay_redelivery_plan_warns_when_partitioned_boundary_is_incomplete() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-redelivery-plan-incomplete-{}",
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
    publish_local_partitioned_transaction(&config, "tx-partitioned-local", true).await;
    let topics = config
        .replay_redelivery_topics()
        .expect("redelivery topics");

    let plan = replay_redelivery_plan(
        &config,
        &topics,
        Some(("tx-partitioned-local", "0/16B6C90")),
    )
    .expect("redelivery plan");

    assert_eq!(
        plan.boundary.as_ref().expect("boundary").status,
        "incomplete_boundary"
    );
    assert!(plan.exact_seek_commands.is_empty());
    assert_eq!(
        plan.warnings,
        vec![
            "local redelivery boundary is incomplete_boundary; exact seek commands were not generated"
                .to_string(),
            "missing message kinds: partition_chunk".to_string()
        ]
    );
    assert!(plan.commands.iter().any(|command| {
        command.contains("trellara.local-source.retail-sales.manifest")
            && command.contains("--next-offset <offset>")
    }));

    std::fs::remove_dir_all(root).expect("cleanup");
}
