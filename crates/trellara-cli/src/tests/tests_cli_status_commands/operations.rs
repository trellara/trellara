use super::*;

#[test]
fn cli_parses_relay_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "relay",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--max-transactions",
        "1",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Relay(RelayArgs {
            max_transactions: 1,
            ..
        })
    ));
}

#[test]
fn cli_parses_verify_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "verify",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--table",
        "public.sales",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Verify(VerifyArgs {
            table: Some(table),
            ..
        }) if table == "public.sales"
    ));
}

#[test]
fn cli_parses_reseed_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "reseed",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--table",
        "public.sales",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Reseed(ReseedArgs {
            table: Some(table),
            ..
        }) if table == "public.sales"
    ));
}

#[test]
fn cli_parses_snapshot_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "snapshot",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--run-id",
        "snapshot-run-1",
        "--table",
        "public.sales",
        "--force",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Snapshot(SnapshotArgs {
            run_id: Some(run_id),
            table: Some(table),
            force: true,
            ..
        }) if run_id == "snapshot-run-1" && table == "public.sales"
    ));
}
