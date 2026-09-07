use super::*;

#[test]
fn cli_parses_inspect_transaction_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "inspect-transaction",
        "--file",
        "tx.pb",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::InspectTransaction(InspectTransactionArgs { file, format })
            if file.as_path() == Path::new("tx.pb")
                && format == TransactionInspectOutputFormat::Text
    ));
}

#[test]
fn cli_parses_partition_local_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "partition-local",
        "--config",
        "examples/retail-fleet/partitioned.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::PartitionLocal(_)));
}

#[test]
fn cli_parses_partition_watermarks_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "partition-watermarks",
        "--config",
        "examples/retail-fleet/partitioned.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PartitionWatermarks(PartitionWatermarksArgs { config, format })
            if config.as_path() == Path::new("examples/retail-fleet/partitioned.yml")
                && format == PilotGuideOutputFormat::Text
    ));
}

#[test]
fn cli_parses_partition_rebalance_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "partition-rebalance-plan",
        "--config",
        "examples/retail-fleet/partitioned.yml",
        "--format",
        "text",
        "--max-skew-percent",
        "25",
        "--plan-moves",
        "--partition-event-count",
        "0=1000",
        "--partition-event-count",
        "1=100",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PartitionRebalancePlan(PartitionRebalancePlanArgs {
            config,
            format,
            max_skew_percent,
            plan_moves,
            partition_event_counts,
        }) if config.as_path() == Path::new("examples/retail-fleet/partitioned.yml")
            && format == PilotGuideOutputFormat::Text
            && max_skew_percent == 25
            && plan_moves
            && partition_event_counts == vec!["0=1000".to_string(), "1=100".to_string()]
    ));
}
