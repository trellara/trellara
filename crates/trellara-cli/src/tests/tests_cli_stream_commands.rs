use super::*;

#[test]
fn cli_parses_stream_inspect_local_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "stream",
        "inspect-local",
        "--config",
        "examples/retail-fleet/local.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Stream {
            command: StreamCommand::InspectLocal(_)
        }
    ));
}

#[test]
fn cli_parses_stream_seek_local_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "stream",
        "seek-local",
        "--config",
        "examples/retail-fleet/local.yml",
        "--topic",
        "trellara.local-source.retail-sales.strict",
        "--next-offset",
        "3",
        "--transaction-id",
        "tx-1",
        "--commit-lsn",
        "0/16B6C50",
        "--consumer-group",
        "applier",
        "--allow-ahead",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Stream {
            command: StreamCommand::SeekLocal(LocalStreamSeekArgs {
                topic,
                next_offset: 3,
                transaction_id: Some(transaction_id),
                commit_lsn: Some(commit_lsn),
                consumer_group: Some(group),
                allow_ahead: true,
                ..
            })
        } if topic == "trellara.local-source.retail-sales.strict"
            && transaction_id == "tx-1"
            && commit_lsn == "0/16B6C50"
            && group == "applier"
    ));
}

#[test]
fn cli_parses_stream_locate_local_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "stream",
        "locate-local",
        "--config",
        "examples/retail-fleet/local.yml",
        "--transaction-id",
        "tx-1",
        "--commit-lsn",
        "0/16B6C50",
        "--topic",
        "trellara.local-source.retail-sales.strict",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Stream {
            command: StreamCommand::LocateLocal(LocalStreamLocateArgs {
                transaction_id,
                commit_lsn: Some(commit_lsn),
                topic: Some(topic),
                ..
            })
        } if transaction_id == "tx-1"
            && commit_lsn == "0/16B6C50"
            && topic == "trellara.local-source.retail-sales.strict"
    ));
}

#[test]
fn cli_parses_stream_reconstruct_local_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "stream",
        "reconstruct-local",
        "--config",
        "examples/retail-fleet/local.yml",
        "--transaction-id",
        "tx-1",
        "--commit-lsn",
        "0/16B6C50",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Stream {
            command: StreamCommand::ReconstructLocal(LocalStreamReconstructArgs {
                transaction_id,
                commit_lsn: Some(commit_lsn),
                ..
            })
        } if transaction_id == "tx-1" && commit_lsn == "0/16B6C50"
    ));
}
