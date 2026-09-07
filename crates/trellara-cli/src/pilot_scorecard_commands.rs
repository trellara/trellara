pub(crate) fn pilot_scorecard_next_commands(
    config_path: &str,
    local_stream: bool,
    has_target: bool,
    partitioned: bool,
) -> Vec<String> {
    let mut next_commands = vec![
        format!("trellara check --config {config_path} --format text"),
        format!("trellara contract-test --config {config_path}"),
        format!("trellara snapshot --config {config_path} --run-id pilot-snapshot-1"),
    ];
    if local_stream && has_target {
        next_commands.push(format!(
            "trellara run --local --verify --format text --config {config_path} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
        ));
    } else if has_target {
        next_commands.push(format!("trellara relay --config {config_path}"));
        next_commands.push(format!("trellara apply --config {config_path}"));
        next_commands.push(format!("trellara verify --config {config_path}"));
    }
    next_commands.push(format!("trellara pilot-package --config {config_path}"));
    next_commands.push(format!(
        "trellara status --config {config_path} --view diagnostics --format text"
    ));
    if partitioned {
        if local_stream {
            next_commands.push(format!(
                "trellara stream reconstruct-local --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
            ));
        }
        next_commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
        next_commands.push(format!(
            "trellara partition-rebalance-plan --config {config_path} --partition-event-count <partition>=<count> --format text"
        ));
    }
    next_commands
}
