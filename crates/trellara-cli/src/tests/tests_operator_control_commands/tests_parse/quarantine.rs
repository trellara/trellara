use super::*;

#[test]
fn cli_parses_quarantine_list_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "quarantine",
        "list",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--limit",
        "5",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Quarantine {
            command: QuarantineCommand::List(QuarantineListArgs { limit: 5, .. })
        }
    ));
}

#[test]
fn cli_parses_quarantine_clear_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "quarantine",
        "clear",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--transaction-id",
        "tx-blocked",
        "--commit-lsn",
        "0/16B6D28",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Quarantine {
            command: QuarantineCommand::Clear(QuarantineClearArgs {
                transaction_id,
                commit_lsn,
                ..
            })
        } if transaction_id == "tx-blocked" && commit_lsn == "0/16B6D28"
    ));
}

#[test]
fn cli_parses_quarantine_replay_ready_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "quarantine",
        "replay-ready",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--transaction-id",
        "tx-blocked",
        "--commit-lsn",
        "0/16B6D28",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Quarantine {
            command: QuarantineCommand::ReplayReady(QuarantineReplayReadyArgs {
                transaction_id,
                commit_lsn,
                ..
            })
        } if transaction_id == "tx-blocked" && commit_lsn == "0/16B6D28"
    ));
}
