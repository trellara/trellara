use super::*;

#[test]
fn flow_create_summary_lists_flow_and_next_commands() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary =
        FlowCreateSummary::from_config(&config, Path::new("examples/retail-fleet/strict.yml"));

    assert!(summary.accepted);
    assert_eq!(summary.source_id, "local-source");
    assert_eq!(summary.dataset_id, "retail-sales");
    assert_eq!(summary.mode, "strict_transaction_order");
    assert_eq!(summary.tables, vec!["public.sales".to_string()]);
    assert_eq!(summary.stream.kind, "kafka");
    assert_eq!(
        summary.stream.primary_topic,
        "trellara.local-source.retail-sales.strict"
    );
    assert_eq!(
            summary.next_commands,
            vec![
                "trellara preflight --config examples/retail-fleet/strict.yml".to_string(),
                "trellara contract-test --config examples/retail-fleet/strict.yml".to_string(),
                "trellara schema-discover --config examples/retail-fleet/strict.yml".to_string(),
                "trellara lake plan --config examples/retail-fleet/strict.yml".to_string(),
                "trellara bootstrap --config examples/retail-fleet/strict.yml".to_string(),
                "trellara relay --config examples/retail-fleet/strict.yml".to_string(),
                "trellara chaos run".to_string(),
                "trellara apply-schema --config examples/retail-fleet/strict.yml".to_string(),
                "trellara apply --config examples/retail-fleet/strict.yml".to_string(),
                "trellara verify --config examples/retail-fleet/strict.yml".to_string(),
                "trellara status --config examples/retail-fleet/strict.yml --view report --format text"
                    .to_string(),
                "trellara status --config examples/retail-fleet/strict.yml --view alerts --format text"
                    .to_string(),
                "trellara status --config examples/retail-fleet/strict.yml --view dashboard --format text"
                    .to_string(),
                "trellara status --config examples/retail-fleet/strict.yml --view metrics"
                    .to_string(),
                "trellara status --config examples/retail-fleet/strict.yml --view diagnostics --format text"
                    .to_string(),
                "trellara check --config examples/retail-fleet/strict.yml".to_string(),
                "trellara repair-plan --config examples/retail-fleet/strict.yml".to_string(),
                "trellara quarantine list --config examples/retail-fleet/strict.yml".to_string(),
                "trellara quarantine replay-ready --config examples/retail-fleet/strict.yml --transaction-id <tx> --commit-lsn <lsn>".to_string(),
                "trellara reseed --config examples/retail-fleet/strict.yml".to_string(),
            ]
        );
}

#[test]
fn flow_create_summary_includes_partition_local_for_partitioned_flows() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary =
        FlowCreateSummary::from_config(&config, Path::new("examples/retail-fleet/partitioned.yml"));

    assert!(summary.next_commands.contains(
        &"trellara partition-local --config examples/retail-fleet/partitioned.yml".to_string()
    ));
    assert!(summary.next_commands.contains(
        &"trellara partition-watermarks --config examples/retail-fleet/partitioned.yml".to_string()
    ));
    assert!(summary.next_commands.iter().any(|command| command.contains(
        "trellara partition-rebalance-plan --config examples/retail-fleet/partitioned.yml"
    )));
}
